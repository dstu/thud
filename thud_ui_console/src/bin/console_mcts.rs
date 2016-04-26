extern crate clap;
#[macro_use] extern crate log;
extern crate mcts;
extern crate rand;
extern crate thud_game;
extern crate thud_ui_common;

use clap::App;
use mcts::Payoff;
use rand::isaac::IsaacRng;
use rand::SeedableRng;

fn main() {
    // Set up arg handling.
    let matches = {
        let app = thud_ui_common::set_args(
            App::new("console_mcts")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Plays out Thud MCTS iterations"),
            &[thud_ui_common::ITERATION_COUNT_FLAG,
              thud_ui_common::SIMULATION_COUNT_FLAG,
              thud_ui_common::EXPLORATION_BIAS_FLAG,
              thud_ui_common::INITIAL_BOARD_FLAG,
              thud_ui_common::INITIAL_PLAYER_FLAG,
              thud_ui_common::LOG_LEVEL_FLAG,
              thud_ui_common::MOVE_SELECTION_CRITERION_FLAG,
              thud_ui_common::COMPACT_SEARCH_GRAPH_FLAG,]);
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
            Some(Ok(x)) => x.cells(),
            Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
            None => thud_game::board::Cells::default(),
        };
    let toggle_initial_player =
        match matches.value_of(thud_ui_common::INITIAL_PLAYER_FLAG).map(|x| x.parse::<thud_game::Role>()) {
            None | Some(Ok(thud_game::Role::Dwarf)) => false,
            Some(Ok(thud_game::Role::Troll)) => true,
            Some(Err(x)) => panic!("{}", x),
        };
    let logging_level =
        match matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).map(|x| x.parse::<log::LogLevelFilter>()) {
            Some(Ok(x)) => x,
            Some(Err(_)) => panic!("Bad logging level '{}'", matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).unwrap()),
            None => log::LogLevelFilter::Info,
        };
    let move_selection_criterion =
        match matches.value_of(thud_ui_common::MOVE_SELECTION_CRITERION_FLAG).map(|x| x.parse::<thud_ui_common::MoveSelectionCriterion>()) {
            Some(Ok(x)) => x,
            Some(Err(e)) => panic!("Bad move selection criterion: {}", e),
            None => thud_ui_common::MoveSelectionCriterion::VisitCount,
        };
    let rng =
        match matches.value_of(thud_ui_common::RNG_SEED_FLAG).map(|x| x.parse::<u32>()) {
            Some(Ok(x)) => IsaacRng::from_seed(&[x]),
            Some(Err(e)) => panic!("Bad RNG seed: {}", e),
            None => IsaacRng::new_unseeded(),
        };
    let compact_graph = matches.is_present(thud_ui_common::COMPACT_SEARCH_GRAPH_FLAG);

    // Set up logging.
    thud_ui_common::init_logger(logging_level);

    // Play game.
    let mut state = thud_ui_common::ThudState::new(initial_cells);
    if toggle_initial_player {
        state.toggle_active_role();
    }
    let mut graph = mcts::new_search_graph::<thud_ui_common::ThudGame>();

    let mut search_state = mcts::SearchState::<IsaacRng, thud_ui_common::ThudGame>::new(rng, exploration_bias);
    search_state.initialize(&mut graph, &state);
    let mut turn_number = 0;
    while !state.terminated() {
        info!("begin turn {}; board: {:?}", turn_number, state);
        if graph.get_node(&state).is_none() {
            error!("board not found in playout graph; reinitializing");
            search_state.initialize(&mut graph, &state);
        }

        // {
        //     let mut available_actions: Vec<game::Action> =
        //         graph.get_node(&state).unwrap().get_child_list().iter()
        //         .map(|c| c.get_data().action).collect();
        //     available_actions.sort_by(thud_game::util::cmp_actions);
        //     let mut state_actions: Vec<game::Action> = state.actions().collect();
        //     state_actions.sort_by(thud_game::util::cmp_actions);
        //     debug!("Checking comparison of state_actions: {:?} vs. available actions: {:?}",
        //            state_actions, available_actions);
        //     assert_eq!(state_actions, available_actions);
        // }

        let mut best_action = None;
        for iteration in 0..iteration_count {
            if iteration % 100 == 0 {
                info!("iteration: {} / {} = {}%", iteration, iteration_count,
                      ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
            }
            match search_state.search(&mut graph, &state,
                                      |_: usize| mcts::SearchSettings {
                                          simulation_count: simulation_count,
                                      }) {
                Ok(stats) => {
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
                        let mut best_visits = ::std::u32::MIN;
                        let mut best_ucb = ::std::f64::MIN;
                        for actions in stats.into_iter() {
                            info!("{:?}: {:?}; UCB = {:?}", actions.action, actions.payoff, actions.ucb);
                            let payoff_ref: &thud_ui_common::ThudPayoff = &actions.payoff;
                            let payoff_visits = payoff_ref.visits();
                            best_action = match (best_action, move_selection_criterion) {
                                (None, _) => Some(actions.action),
                                (Some(_), thud_ui_common::MoveSelectionCriterion::VisitCount) if best_visits < payoff_visits => {
                                    best_visits = payoff_visits;
                                    Some(actions.action)
                                },
                                (Some(_), thud_ui_common::MoveSelectionCriterion::Ucb) =>
                                    match actions.ucb {
                                        Ok(mcts::UcbValue::Value(x)) if best_ucb < x => {
                                            best_ucb = x;
                                            Some(actions.action)
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
                let mut canonical_state = graph.get_node(&state).unwrap().get_label().clone();
                canonical_state.do_action(&action);
                state.set_from_convolved(&canonical_state);
                if compact_graph {
                    debug!("collecting garbage and compacting search graph");
                    graph.retain_reachable_from(&[&canonical_state]);
                    debug!("done compacting search graph");
                } else {
                    debug!("potential memory leak: not compatcing search graph");
                }
                turn_number += 1;
            },
            None => {
                info!("turn {}: no best action. exiting.", turn_number);
                break
            }
        }
    }
    info!("game over. final board state: {:?}", state);
}
