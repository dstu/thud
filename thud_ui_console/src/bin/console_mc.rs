extern crate clap;
#[macro_use]
extern crate log;
extern crate mcts;
extern crate rand;
extern crate thud_game;
extern crate thud_ui_common;

use clap::App;
use mcts::{Payoff, Statistics};

use std::default::Default;

fn main() {
  // Set up arg handling.
  let matches = {
    let app = thud_ui_common::set_args(
      App::new("console_mc")
        .version("0.1.0")
        .author("Stu Black <trurl@freeshell.org>")
        .about("Play out Thud Monte Carlo iterations"),
      &[
        thud_ui_common::ITERATION_COUNT_FLAG,
        thud_ui_common::SIMULATION_COUNT_FLAG,
        thud_ui_common::EXPLORATION_BIAS_FLAG,
        thud_ui_common::INITIAL_BOARD_FLAG,
        thud_ui_common::INITIAL_PLAYER_FLAG,
        thud_ui_common::LOG_LEVEL_FLAG,
      ],
    );
    app.get_matches()
  };

  let iteration_count = match matches
    .value_of(thud_ui_common::ITERATION_COUNT_FLAG)
    .unwrap()
    .parse::<usize>()
  {
    Ok(x) => x,
    Err(e) => panic!("Bad iteration count: {}", e),
  };
  let simulation_count = match matches
    .value_of(thud_ui_common::SIMULATION_COUNT_FLAG)
    .unwrap()
    .parse::<usize>()
  {
    Ok(x) => x,
    Err(e) => panic!("Bad simulation count: {}", e),
  };
  let exploration_bias = match matches
    .value_of(thud_ui_common::EXPLORATION_BIAS_FLAG)
    .unwrap()
    .parse::<f64>()
  {
    Ok(x) => x,
    Err(e) => panic!("Bad exploration bias: {}", e),
  };
  let initial_cells = match matches
    .value_of(thud_ui_common::INITIAL_BOARD_FLAG)
    .map(|x| x.parse::<thud_ui_common::InitialBoard>())
  {
    None => thud_game::board::Cells::default(),
    Some(Ok(x)) => x.cells(),
    Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
  };
  let toggle_initial_player = match matches
    .value_of(thud_ui_common::INITIAL_PLAYER_FLAG)
    .map(|x| x.parse::<thud_game::Role>())
  {
    None | Some(Ok(thud_game::Role::Dwarf)) => false,
    Some(Ok(thud_game::Role::Troll)) => true,
    Some(Err(x)) => panic!("{}", x),
  };
  let logging_level = match matches
    .value_of(thud_ui_common::LOG_LEVEL_FLAG)
    .map(|x| x.parse::<log::LogLevelFilter>())
  {
    Some(Ok(x)) => x,
    Some(Err(_)) => panic!(
      "Bad logging level '{}'",
      matches.value_of(thud_ui_common::LOG_LEVEL_FLAG).unwrap()
    ),
    None => log::LogLevelFilter::Info,
  };

  // Set up logging.
  thud_ui_common::init_logger(logging_level);

  let mut state = <thud_ui_common::ThudGame as mcts::Game>::State::new(initial_cells);
  if toggle_initial_player {
    state.toggle_active_role();
  }

  let mut rng = rand::thread_rng();
  let mut turn_number = 0;
  loop {
    info!("begin turn {}; board: {:?}", turn_number, state);
    let actions: Vec<thud_game::Action> = state.role_actions(*state.active_role()).collect();
    if actions.is_empty() {
      info!("No actions available. Exiting.");
      return;
    }
    let action_statistics: Vec<thud_ui_common::ThudStatistics> = {
      let mut v = Vec::with_capacity(actions.len());
      for _ in 0..actions.len() {
        v.push(Default::default());
      }
      v
    };
    let mut log_iteration = 0.0;
    for iteration in 0..iteration_count {
      if iteration % 100 == 0 {
        info!(
          "iteration: {} / {} = {:.02}%",
          iteration,
          iteration_count,
          ((100 * iteration) as f64) / (iteration_count as f64)
        );
      }
      let mut selected_action_index = 0;
      let mut best_ucb = std::f64::MIN;
      for (i, stats) in action_statistics.iter().enumerate() {
        let payoff = stats.as_payoff();
        if payoff.visits() == 0 {
          selected_action_index = i;
          best_ucb = std::f64::MAX;
          break;
        } else {
          let child_visits = payoff.visits() as f64;
          let child_payoff = payoff.score(state.active_role()) as f64;
          let ucb = child_payoff / child_visits
            + exploration_bias * f64::sqrt(log_iteration / child_visits);
          // TODO tie-breaking.
          if ucb > best_ucb {
            best_ucb = ucb;
            selected_action_index = i;
          }
        }
      }
      trace!(
        "UCB selected action {:?} [UCB = {}]",
        actions[selected_action_index],
        best_ucb
      );
      for _ in 0..simulation_count {
        let payoff = mcts::simulate::simulate::<rand::ThreadRng, thud_ui_common::ThudGame>(
          &mut state.clone(),
          &mut rng,
        );
        trace!("simulated payoff {:?}", payoff);
        action_statistics[selected_action_index].increment(&payoff);
      }
      log_iteration = f64::ln((iteration + 1) as f64);
    }

    let mut most_visited_index = std::usize::MIN;
    let mut most_visits = 0;
    for (i, stats) in action_statistics.iter().enumerate() {
      let payoff = stats.as_payoff();
      info!("Action {:?} gets {:?}", actions[i], payoff);
      // TODO tie-breaking.
      if payoff.visits() > most_visits {
        most_visited_index = i;
        most_visits = payoff.visits();
      }
    }
    info!(
      "Performing move {:?} with {:?}",
      actions[most_visited_index], action_statistics[most_visited_index]
    );
    state.do_action(&actions[most_visited_index]);
    turn_number += 1;
  }
}
