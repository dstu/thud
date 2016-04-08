extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rand;
extern crate thud;

use ::clap::App;

use ::thud::game;
use ::thud::mcts;
use ::thud::util;

fn main() {
    // Set up arg handling.
    let matches = {
        let app = util::set_common_args(
            App::new("console_mcts")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Plays out Thud MCTS iterations"),
            &[util::ITERATION_COUNT_FLAG,
              util::SIMULATION_COUNT_FLAG,
              util::EXPLORATION_BIAS_FLAG,
              util::INITIAL_BOARD_FLAG,
              util::INITIAL_PLAYER_FLAG,
              util::LOG_LEVEL_FLAG,
              util::MOVE_SELECTION_CRITERION_FLAG]);
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
            Some(Ok(x)) => x.cells(),
            Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
            None => game::board::Cells::default(),
        };
    let toggle_initial_player =
        match matches.value_of(util::INITIAL_PLAYER_FLAG).map(|x| x.parse::<game::Role>()) {
            None | Some(Ok(game::Role::Dwarf)) => false,
            Some(Ok(game::Role::Troll)) => true,
            Some(Err(x)) => panic!("{}", x),
        };
    let logging_level =
        match matches.value_of(util::LOG_LEVEL_FLAG).map(|x| x.parse::<log::LogLevelFilter>()) {
            Some(Ok(x)) => x,
            Some(Err(_)) => panic!("Bad logging level '{}'", matches.value_of(util::LOG_LEVEL_FLAG).unwrap()),
            None => log::LogLevelFilter::Info,
        };
    let move_selection_criterion =
        match matches.value_of(util::MOVE_SELECTION_CRITERION_FLAG).map(|x| x.parse::<util::MoveSelectionCriterion>()) {
            Some(Ok(x)) => x,
            Some(Err(e)) => panic!("Bad move selection criterion: {}", e),
            None => util::MoveSelectionCriterion::VisitCount,
        };

    // Set up logging.
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %T%.3f%z").to_string(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: logging_level,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }

    // Play game.
    let mut state = mcts::State::new(initial_cells);
    if toggle_initial_player {
        state.toggle_active_role();
    }
    let mut graph = mcts::Graph::new();

    util::initialize_search(state.clone(), &mut graph);
    let mut search_state = mcts::SearchState::new(rand::thread_rng(), exploration_bias);
    let mut turn_number = 0;
    loop {
        info!("begin turn {}; board: {}", turn_number, ::thud::game::board::format_board(state.board()));
        let mut best_action = None;
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
                        for &(_, stats, _) in stats.iter() {
                            count += stats.visits;
                        }
                        count
                    };
                    trace!("total visits at top level: {}", toplevel_visits);
                    // TODO: this commented-out block is only valid when we haven't
                    // propagated a multi-visit payoff upwards from conneting to an
                    // extant vertex.
                    // if simulation_count == 1 && toplevel_visits != iteration + 1 {
                    //     console_ui::write_board(state.board());
                    //     for (action, stats) in stats.into_iter() {
                    //         println!("{:?}: [{}, {}] = {:?} / {}", action,
                    //                  (stats.payoff.values[0] as f64) / (stats.visits as f64),
                    //                  (stats.payoff.values[1] as f64) / (stats.visits as f64),
                    //                  stats.payoff, stats.visits);
                    //     }
                    //     panic!("total visits at top level is {}, but iteration count is {}",
                    //            toplevel_visits, iteration)
                    // }
                    if iteration % 1000 == 0 || iteration + 1 == iteration_count {
                        info!("root stats:");
                        let mut best_visits = ::std::usize::MIN;
                        let mut best_ucb = ::std::f64::MIN;
                        for (action, stats, ucb) in stats.into_iter() {
                            info!("{:?}: [{}, {}] = {:?} / {}; UCB = {:?}", action,
                                  (stats.payoff.values[0] as f64) / (stats.visits as f64),
                                  (stats.payoff.values[1] as f64) / (stats.visits as f64),
                                  stats.payoff, stats.visits, ucb);
                            best_action = match (best_action, move_selection_criterion) {
                                (None, _) => Some(action),
                                (Some(_), util::MoveSelectionCriterion::VisitCount) if best_visits < stats.visits => {
                                    best_visits = stats.visits;
                                    Some(action)
                                },
                                (Some(_), util::MoveSelectionCriterion::Ucb) =>
                                    match ucb {
                                        Ok(mcts::UcbProxy::Value(x)) if best_ucb < x => {
                                            best_ucb = x;
                                            Some(action)
                                        },
                                        _ => best_action,
                                    },
                                _ => best_action,
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Error in seach iteration {}: {}", iteration, e);
                    break
                },
            }
        }

        // Finish turn.
        match best_action {
            Some(action) => {
                info!("turn {}: performing best action {:?}", turn_number, action);
                state.do_action(&action);
                graph = mcts::Graph::new();
                util::initialize_search(state.clone(), &mut graph);
                turn_number += 1;
            },
            None => {
                info!("turn {}: no best action. exiting.", turn_number);
                break
            }
        }
    }
    // console_ui::write_search_graph(&graph, &state);
}
