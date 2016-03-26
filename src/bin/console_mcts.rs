use std::str::FromStr;

extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rand;
extern crate thud;

use ::clap::App;

use ::thud::console_ui;
use ::thud::game;
use ::thud::game::board;
use ::thud::mcts;

pub fn initialize_search(state: game::State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_player().role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}

fn main() {
    // Set up logging.
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", chrono::Local::now().to_rfc3339(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }

    // Set up arg handling.
    let matches = {
        let mut app = ::thud::util::set_common_args(
            App::new("console_mcts")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Plays out Thud MCTS iterations"));
        app.get_matches()
    };
    let iteration_count =
        match matches.value_of(::thud::util::ITERATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad iteration count: {}", e),
        };
    let simulation_count =
        match matches.value_of(::thud::util::SIMULATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad simulation count: {}", e),
        };
    let exploration_bias =
        match matches.value_of(::thud::util::EXPLORATION_BIAS_FLAG).unwrap().parse::<f64>() {
            Ok(x) => x,
            Err(e) => panic!("Bad exploration bias: {}", e),
        };

    // Play game.
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();

    initialize_search(state.clone(), &mut graph);
    let mut search_state = mcts::SearchState::new(rand::thread_rng(), exploration_bias);
    for iteration in 0..iteration_count {
        if iteration % 100 == 0 {
            info!("iteration: {} / {} = {}%", iteration, iteration_count,
                  ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
        }
        match search_state.search(&mut graph, state.clone(),
                                  |_: usize| mcts::SearchSettings {
                                      simulation_count: simulation_count,
                                  }) {
            Ok(stats) => {
                let toplevel_visits = {
                    let mut count = 0;
                    for &(_, stats) in stats.iter() {
                        count += stats.visits;
                    }
                    count
                };
                trace!("total visits at top level: {}", toplevel_visits);
                if simulation_count == 1 && toplevel_visits != iteration + 1 {
                    console_ui::write_board(state.board());
                    for (action, stats) in stats.into_iter() {
                        println!("{:?}: [{}, {}] = {:?} / {}", action,
                                 (stats.payoff.values[0] as f64) / (stats.visits as f64),
                                 (stats.payoff.values[1] as f64) / (stats.visits as f64),
                                 stats.payoff, stats.visits);
                    }
                    panic!("total visits at top level is {}, but iteration count is {}",
                           toplevel_visits, iteration)
                }
                if iteration % 1000 == 0 || iteration + 1 == iteration_count {
                    println!("root stats:");
                    for (action, stats) in stats.into_iter() {
                        println!("{:?}: [{}, {}] = {:?} / {}", action,
                                 (stats.payoff.values[0] as f64) / (stats.visits as f64),
                                 (stats.payoff.values[1] as f64) / (stats.visits as f64),
                                 stats.payoff, stats.visits);
                    }
                }
            },
            Err(e) => {
                error!("Error in seach iteration {}: {}", iteration, e);
                break
            },
        }
    }
    // console_ui::write_search_graph(&graph, &state);
}
