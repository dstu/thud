use mcts;
use rand::Rng;
use std::marker::Send;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/*
pub const FLAG_ITERATION_COUNT: &'static str = "iterations";
pub const FLAG_SIMULATION_COUNT: &'static str = "simulations";
pub const FLAG_EXPLORATION_BIAS: &'static str = "explore_bias";
pub const FLAG_AI_PLAYER: &'static str = "ai_player";
pub const FLAG_MOVE_SELECTION_CRITERION: &'static str = "move_selection_criterion";
pub const FLAG_RNG_SEED: &'static str = "rng_seed";
pub const FLAG_COMPACT_SEARCH_GRAPH: &'static str = "compact_search_graph";

      x if x == FLAG_ITERATION_COUNT => clap::Arg::with_name(ITERATION_COUNT_FLAG)
        .short("i")
        .long("iterations")
        .takes_value(true)
        .value_name("ITERATIONS")
        .help("Number of Monte Carlo iterations to run per epoch")
        .required(true),
      x if x == FLAG_SIMULATION_COUNT => clap::Arg::with_name(SIMULATION_COUNT_FLAG)
        .short("s")
        .long("simulations")
        .takes_value(true)
        .value_name("SIMULATIONS")
        .help("Number of simulations to run at each expansion step")
        .required(true),
      x if x == FLAG_EXPLORATION_BIAS => clap::Arg::with_name(EXPLORATION_BIAS_FLAG)
        .short("b")
        .long("exploration_bias")
        .takes_value(true)
        .value_name("BIAS")
        .help("Exploration bias for UCB computation")
        .required(true),
      x if x == FLAG_MOVE_SELECTION_CRITERION => clap::Arg::with_name(MOVE_SELECTION_CRITERION_FLAG)
        .long("move_selection")
        .takes_value(true)
        .possible_values(&["visitcount", "ucb"])
        .help("Criteria for selecting the best move to make"),
      x if x == FLAG_RNG_SEED => clap::Arg::with_name(RNG_SEED_FLAG)
        .long("seed")
        .takes_value(true)
        .help("Manually specify RNG seed"),
      x if x == FLAG_COMPACT_SEARCH_GRAPH => clap::Arg::with_name(COMPACT_SEARCH_GRAPH_FLAG)
        .long("compact_graph")
        .takes_value(false)
        .help("Compact the search graph after each move"),

*/

pub enum Instruction {
  Exit,
  Search {
    state: crate::State,
    iteration_count: u32,
    simulation_count: usize,
  },
}

pub struct Handle {
  status: Arc<Mutex<Status>>,
  join_handle: thread::JoinHandle<()>,
  parameters_tx: mpsc::SyncSender<Instruction>,
  result_rx: mpsc::Receiver<Vec<mcts::ActionStatistics<crate::Game>>>,
}

#[derive(Clone)]
pub enum Status {
  /// Waiting for a new set of search parameters.
  Waiting,
  /// Initializing search.
  Initializing,
  /// Computing search.
  Working {
    /// Statistics returned from last search iteration.
    statistics: Vec<mcts::ActionStatistics<crate::Game>>,
    /// Iteration being computed (0-based).
    iteration: u32,
    /// Total iterations to complete.
    iteration_count: u32,
  },
  /// Error occurred during search.
  Error(mcts::SearchError),
  /// Thread has exited.
  Done,
}

impl Handle {
  pub fn new<R>(rng: R, exploration_bias: f64) -> Self
  where
    R: 'static + Rng + Send,
  {
    let (parameters_tx, parameters_rx) = mpsc::sync_channel(0);
    let (result_tx, result_rx) = mpsc::sync_channel(0);
    let status = Arc::new(Mutex::new(Status::Waiting));
    let join_handle = {
      let status = status.clone();
      thread::spawn(move || {
        let mut graph = mcts::new_search_graph::<crate::Game>();
        let mut search_state = mcts::SearchState::<R, crate::Game>::new(rng, exploration_bias);
        // Read search parameters (blocks until an instruction is provided).
        while let Instruction::Search {
          state,
          iteration_count,
          simulation_count,
        } = parameters_rx.recv().unwrap()
        {
          // Initialize search.
          *status.lock().unwrap() = Status::Initializing;
          search_state.initialize(&mut graph, &state);
          // First search iteration.
          let mut error = false;
          match search_state.search(&mut graph, &state, |_| mcts::SearchSettings {
            simulation_count: simulation_count,
          }) {
            Ok(stats) => {
              *status.lock().unwrap() = Status::Working {
                statistics: stats,
                iteration: 1,
                iteration_count: iteration_count,
              }
            }
            Err(e) => {
              *status.lock().unwrap() = Status::Error(e);
              error = true;
            }
          }
          // Other search iterations.
          for iteration in 1..iteration_count {
            if error {
              break;
            }
            match search_state.search(&mut graph, &state, |_| mcts::SearchSettings {
              simulation_count: simulation_count,
            }) {
              Ok(stats) => {
                *status.lock().unwrap() = Status::Working {
                  statistics: stats,
                  iteration: iteration,
                  iteration_count: iteration_count,
                }
              }
              Err(e) => {
                *status.lock().unwrap() = Status::Error(e);
                error = true;
              }
            }
          }
          if !error {
            // Write search result (blocks until someone consumes it).
            let mut status = status.lock().unwrap();
            let statistics = if let Status::Working { ref statistics, .. } = *status {
              statistics.clone()
            } else {
              panic!("Unable to determine search result");
            };
            result_tx.send(statistics);
          }
        }
        *status.lock().unwrap() = Status::Done;
      })
    };
    Handle {
      status: Arc::new(Mutex::new(Status::Waiting)),
      join_handle: join_handle,
      parameters_tx: parameters_tx,
      result_rx: result_rx,
    }
  }

  pub fn status(&self) -> Status {
    (*self.status.lock().unwrap()).clone()
  }

  pub fn join(self) -> thread::Result<()> {
    self.join_handle.join()
  }
}
