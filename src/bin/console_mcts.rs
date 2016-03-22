use std::str::FromStr;

extern crate chrono;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rand;
extern crate thud;

use ::thud::console_ui;
use ::thud::game;
use ::thud::game::board;
use ::thud::mcts;

use std::env::args;

pub fn initialize_search(state: game::State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_player().role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}

fn main() {
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
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();

    let iteration_count = args().skip(1).next().unwrap().parse::<usize>().ok().unwrap();
    initialize_search(state.clone(), &mut graph);
    let mut search_state = mcts::SearchState::new(rand::thread_rng(), 0.1);
    for iteration in 0..iteration_count {
        if iteration % 1000 == 0 {
            info!("iteration: {} / {} = {}%", iteration, iteration_count,
                  ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
        }
        match search_state.search(&mut graph, state.clone()) {
            Ok(stats) => {
                let toplevel_visits = {
                    let mut count = 0;
                    for &(_, stats) in stats.iter() {
                        count += stats.visits;
                    }
                    count
                };
                trace!("total visits at top level: {}", toplevel_visits);
                if toplevel_visits != iteration + 1 {
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
