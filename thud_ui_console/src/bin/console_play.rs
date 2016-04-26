extern crate clap;
#[macro_use] extern crate log;
extern crate mcts;
extern crate rand;
extern crate thud_game;
extern crate thud_ui_common;
extern crate thud_ui_console;

use clap::App;
use mcts::{Payoff, State};

use std::default::Default;

fn main() {
    // Set up arg handling.
    let matches = {
        let app = thud_ui_common::set_args(
            App::new("console_play")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Play against Thud AI"),
            &[thud_ui_common::ITERATION_COUNT_FLAG,
              thud_ui_common::SIMULATION_COUNT_FLAG,
              thud_ui_common::EXPLORATION_BIAS_FLAG,
              thud_ui_common::INITIAL_BOARD_FLAG,
              thud_ui_common::AI_PLAYER_FLAG,
              thud_ui_common::LOG_LEVEL_FLAG]);
        app.get_matches()
    };
    let iteration_count =
        match matches.value_of(thud_ui_common::ITERATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad iteration count: {}", e),
        };
    let simulation_count =
        match matches.value_of(thud_ui_common::SIMULATION_COUNT_FLAG).unwrap().parse::<usize>() {
            Ok(x) => x,
            Err(e) => panic!("Bad simulation count: {}", e),
        };
    let exploration_bias =
        match matches.value_of(thud_ui_common::EXPLORATION_BIAS_FLAG).unwrap().parse::<f64>() {
            Ok(x) => x,
            Err(e) => panic!("Bad exploration bias: {}", e),
        };
    let initial_cells =
        match matches.value_of(thud_ui_common::INITIAL_BOARD_FLAG).map(|x| x.parse::<thud_ui_common::InitialBoard>()) {
            None => thud_game::board::Cells::default(),
            Some(Ok(x)) => x.cells(),
            Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
        };
    let ai_role = match matches.value_of(thud_ui_common::AI_PLAYER_FLAG).map(|x| x.parse::<thud_game::Role>()) {
        None | Some(Ok(thud_game::Role::Dwarf)) => thud_game::Role::Dwarf,
        Some(Ok(thud_game::Role::Troll)) => thud_game::Role::Troll,
        Some(Err(x)) => panic!("{}", x),
    };
    let human_role = ai_role.toggle();
    let logging_level =
        match matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).map(|x| x.parse::<log::LogLevelFilter>()) {
            Some(Ok(x)) => x,
            Some(Err(_)) => panic!("Bad logging level '{}'", matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).unwrap()),
            None => log::LogLevelFilter::Info,
        };

    // Set up logging.
    thud_ui_common::init_logger(logging_level);

    let mut state = thud_ui_common::ThudState::new(initial_cells);
    let mut graph = mcts::new_search_graph::<thud_ui_common::ThudGame>();
    let mut search_state = mcts::SearchState::<rand::ThreadRng, thud_ui_common::ThudGame>::new(rand::thread_rng(), exploration_bias);
    search_state.initialize(&mut graph, &state);
    loop {
        println!("{:?}", state);
        if *state.active_role() == ai_role {
            println!("{:?} player's turn. Thinking...", ai_role);
            if graph.get_node(&state).is_none() {
                search_state.initialize(&mut graph, &state);
                info!("Manually added current game state to graph");
            }
            let mut best_action = None;
            for iteration in 0..iteration_count {
                if iteration % 100 == 0 {
                    info!("iteration: {} / {} = {:.02}%", iteration, iteration_count,
                          ((100 * iteration) as f64) / (iteration_count as f64));
                }
                if iteration == 0 {
                    info!("initial root parents:");
                    for parent in graph.get_node(&state).unwrap().get_parent_list().iter() {
                        info!("{:?}", parent.get_data());
                    }
                    info!("initial root children:");
                    for child in graph.get_node(&state).unwrap().get_child_list().iter() {
                        info!("{:?}", child.get_data());
                    }
                }
                match search_state.search(
                    &mut graph, &state,
                    |_: usize| mcts::SearchSettings {
                        simulation_count: simulation_count,
                    }) {
                    Ok(stats) => {
                        if iteration % 1000 == 0 || iteration + 1 == iteration_count {
                            info!("root stats:");
                            let mut best_visits = ::std::u32::MIN;
                            for actions in stats.into_iter() {
                                info!("{:?}: {:?}; UCB = {:?}", actions.action, actions.payoff, actions.ucb);
                                let payoff_ref: &thud_ui_common::ThudPayoff = &actions.payoff;
                                let payoff_visits = payoff_ref.visits();
                                best_action = match best_action {
                                    None => Some(actions.action),
                                    Some(_) if best_visits < payoff_visits => {
                                        best_visits = payoff_visits;
                                        Some(actions.action)
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
                    graph.retain_reachable_from(&[&state]);
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
                let c = thud_ui_console::prompt_for_piece(state.cells(), human_role);
                let piece_actions: Vec<thud_game::Action> = state.position_actions(c).collect();
                if piece_actions.is_empty() {
                    println!("Piece at {:?} has no actions.", c);
                } else {
                    if let Some(action) = thud_ui_console::select_one(&piece_actions) {
                        let mut moved_state = state.clone();
                        moved_state.do_action(&action);
                        println!("After action, board: {}",
                                 thud_game::board::format_board(moved_state.cells()));
                        println!("Is this okay?");
                        match thud_ui_console::select_one(&["y", "n"]) {
                            Some(&"y") => {
                                state = moved_state;
                                graph.retain_reachable_from(&[&state]);
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
