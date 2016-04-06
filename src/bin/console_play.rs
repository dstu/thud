extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use] extern crate log;
extern crate rand;
extern crate thud;

use clap::App;
use thud::game;
use thud::mcts;
use thud::console_ui;
use thud::util;

pub fn initialize_search(state: mcts::State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}

fn main() {
    // Set up arg handling.
    let matches = {
        let app = util::set_common_args(
            App::new("console_play")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Play against Thud AI"));
        app.get_matches()
    };
    let iteration_count =
        match matches.value_of(util::ITERATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad iteration count: {}", e),
        };
    let simulation_count =
        match matches.value_of(util::SIMULATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad simulation count: {}", e),
        };
    let exploration_bias =
        match matches.value_of(util::EXPLORATION_BIAS_FLAG).unwrap().parse::<f64>() {
            Ok(x) => x,
            Err(e) => panic!("Bad exploration bias: {}", e),
        };
    let initial_cells =
        match matches.value_of(util::INITIAL_BOARD_FLAG).map(|x| x.parse::<util::InitialBoard>()) {
            None => game::board::Cells::default(),
            Some(Ok(x)) => x.cells(),
            Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
        };
    let ai_role = match matches.value_of(util::INITIAL_PLAYER_FLAG).map(|x| x.parse::<game::Role>()) {
        None | Some(Ok(game::Role::Dwarf)) => game::Role::Dwarf,
        Some(Ok(game::Role::Troll)) => game::Role::Troll,
        Some(Err(x)) => panic!("{}", x),
    };
    let human_role = ai_role.toggle();
    let logging_level =
        match matches.value_of(::thud::util::LOG_LEVEL_FLAG).map(|x| x.parse::<log::LogLevelFilter>()) {
            Some(Ok(x)) => x,
            Some(Err(_)) => panic!("Bad logging level '{}'", matches.value_of(::thud::util::LOG_LEVEL_FLAG).unwrap()),
            None => log::LogLevelFilter::Info,
        };

    // Set up logging.
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", chrono::Local::now().to_rfc3339(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: logging_level,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }

    let mut state = mcts::State::new(initial_cells);
    loop {
        console_ui::write_board(state.board());
        if state.active_role() == ai_role {
            let mut best_action = None;
            let mut graph = mcts::Graph::new();
            initialize_search(state.clone(), &mut graph);
            let mut search_state = mcts::SearchState::new(rand::thread_rng(), exploration_bias);
            for iteration in 0..iteration_count {
                if iteration % 100 == 0 {
                    info!("iteration: {} / {} = {:.02}%", iteration, iteration_count,
                          ((100 * iteration) as f64) / (iteration_count as f64));
                }
                match search_state.search(
                    &mut graph, state.clone(),
                    |_: usize| mcts::SearchSettings {
                        simulation_count: simulation_count,
                    }) {
                    Ok(stats) => {
                        if iteration % 1000 == 0 || iteration + 1 == iteration_count {
                            info!("root stats:");
                            let mut best_visits = ::std::usize::MIN;
                            for (action, stats, ucb) in stats.into_iter() {
                                info!("{:?}: [{}, {}] = {:?} / {}; UCB = {:?}", action,
                                      (stats.payoff.values[0] as f64) / (stats.visits as f64),
                                      (stats.payoff.values[1] as f64) / (stats.visits as f64),
                                      stats.payoff, stats.visits, ucb);
                                best_action = match best_action {
                                    None => Some(action),
                                    Some(_) if best_visits < stats.visits => {
                                        best_visits = stats.visits;
                                        Some(action)
                                    },
                                    _ => best_action,
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error in search: {}", e);
                        break
                    }
                }
            }
            match best_action {
                Some(a) => {
                    info!("best action: {:?}", a);
                    state.do_action(&a);
                },
                None => {
                    info!("No best action. Exiting.");
                    return
                }
            }
        } else {
            // Prompt for play.
            loop {
                println!("{:?} player's turn. Enter coordinate of piece to move.", human_role);
                let c = console_ui::prompt_for_piece(state.board(), human_role);
                let piece_actions: Vec<game::Action> = state.position_actions(c).collect();
                if piece_actions.is_empty() {
                    println!("Piece at {:?} has no actions.", c);
                } else {
                    if let Some(action) = console_ui::select_one(&piece_actions) {
                        let mut moved_state = state.clone();
                        moved_state.do_action(&action);
                        println!("After action, board: {}",
                                 game::board::format_board(moved_state.board()));
                        println!("Is this okay?");
                        match console_ui::select_one(&["y", "n"]) {
                            Some(&"y") => {
                                state = moved_state;
                                break
                            },
                            _ => continue,
                        }
                    }
                }
            }
        }
    }
}
