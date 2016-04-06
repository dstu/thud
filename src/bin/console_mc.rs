extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use] extern crate log;
extern crate rand;
extern crate thud;

use clap::App;
use thud::game;
use thud::mcts;
use thud::util;

use std::default::Default;

fn main() {
    // Set up arg handling.
    let matches = {
        let app = util::set_common_args(
            App::new("console_mcs")
                .version("0.1.0")
                .author("Stu Black <trurl@freeshell.org>")
                .about("Play out Thud Monte Carlo iterations"),
            &[util::ITERATION_COUNT_FLAG,
              util::SIMULATION_COUNT_FLAG,
              util::EXPLORATION_BIAS_FLAG,
              util::INITIAL_BOARD_FLAG,
              util::INITIAL_PLAYER_FLAG,
              util::LOG_LEVEL_FLAG]);
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
        match matches.value_of(util::INITIAL_BOARD_FLAG).unwrap().parse::<util::InitialBoard>() {
            Ok(x) => x.cells(),
            Err(e) => panic!("Bad initial board configuration: {}", e),
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

    let mut state = game::State::<game::board::TranspositionalEquivalence>::new(initial_cells);
    if toggle_initial_player {
        state.toggle_active_role();
    }

    let mut rng = rand::thread_rng();
    let mut turn_number = 0;
    loop {
        info!("begin turn {}; board: {}", turn_number, game::board::format_board(state.board()));
        let actions: Vec<game::Action> = state.role_actions(state.active_role()).collect();
        if actions.is_empty() {
            info!("No actions available. Exiting.");
            return
        }
        let mut action_statistics: Vec<mcts::Statistics> = {
            let mut v = Vec::with_capacity(actions.len());
            for _ in 0..actions.len() {
                v.push(Default::default());
            }
            v
        };
        let mut log_iteration = 0.0;
        for iteration in 0..iteration_count {
            if iteration % 100 == 0 {
                info!("iteration: {} / {} = {:.02}%", iteration, iteration_count,
                      ((100 * iteration) as f64) / (iteration_count as f64));
            }
            let mut selected_action_index = 0;
            let mut best_ucb = std::f64::MIN;
            for (i, stats) in action_statistics.iter().enumerate() {
                if stats.visits == 0 {
                    selected_action_index = i;
                    best_ucb = std::f64::MAX;
                    break
                } else {
                    let child_visits = stats.visits as f64;
                    let child_payoff = stats.payoff.score(state.active_role()) as f64;
                    let ucb = child_payoff / child_visits
                        + exploration_bias * f64::sqrt(log_iteration / child_visits);
                    // TODO tie-breaking.
                    if ucb > best_ucb {
                        best_ucb = ucb;
                        selected_action_index = i;
                    }
                }
            }
            trace!("UCB selected action {:?} [UCB = {}]", actions[selected_action_index], best_ucb);
            for _ in 0..simulation_count {
                let payoff = mcts::expand::simulate(&mut state.clone(), &mut rng);
                trace!("simulated payoff {:?}", payoff);
                action_statistics[selected_action_index].increment_visit(payoff);
            }
            log_iteration = f64::ln((iteration + 1) as f64);
        }

        let mut most_visited_index = std::usize::MIN;
        let mut most_visits = 0;
        for (i, stats) in action_statistics.iter().enumerate() {
            info!("Action {:?} gets statistics {:?}", actions[i], stats);
            // TODO tie-breaking.
            if stats.visits > most_visits {
                most_visited_index = i;
                most_visits = stats.visits;
            }
        }
        info!("Performing move {:?} with statistics {:?}",
              actions[most_visited_index], action_statistics[most_visited_index]);
        state.do_action(&actions[most_visited_index]);
        turn_number += 1;
    }
}
