use mcts;
use rand::Rng;
use std::marker::Send;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use {ThudGame, ThudState};

pub enum Instruction {
  Exit,
  Search {
    state: ThudState,
    iteration_count: u32,
    simulation_count: usize,
  },
}

pub struct Handle {
  status: Arc<Mutex<Status>>,
  join_handle: thread::JoinHandle<()>,
  parameters_tx: mpsc::SyncSender<Instruction>,
  result_rx: mpsc::Receiver<Vec<mcts::ActionStatistics<ThudGame>>>,
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
    statistics: Vec<mcts::ActionStatistics<ThudGame>>,
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
        let mut graph = mcts::new_search_graph::<ThudGame>();
        let mut search_state = mcts::SearchState::<R, ThudGame>::new(rng, exploration_bias);
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
