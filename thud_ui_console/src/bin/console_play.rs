use clap::App;
use std::default::Default;
use thud_game;
use thud_ui_common;
use thud_ui_console;

fn main() {
  let mut agents = thud_ui_common::agent_registry::AgentRegistry::new();
  agents
    .register(Box::new(
      thud_ui_common::agent_registry::StdinAgentBuilder::new(),
    ))
    .register(Box::new(
      thud_ui_common::agent_registry::mcts::MctsAgentBuilder::new("mcts1"),
    ))
    .register(Box::new(
      thud_ui_common::agent_registry::mcts::MctsAgentBuilder::new("mcts2"),
    ))
    .register(Box::new(
      thud_ui_common::agent_registry::FileAgentBuilder::new("file_agent"),
    ));

  // Set up arg handling.
  let matches = {
    let mut app = thud_ui_common::set_args(
      App::new("console_play")
        .version("0.1.0")
        .author("Stu Black <trurl@freeshell.org>")
        .about("Play Thud on the console"),
      &[
        thud_ui_common::FLAG_INITIAL_BOARD,
        thud_ui_common::FLAG_INITIAL_PLAYER,
        thud_ui_common::FLAG_LOG_LEVEL,
      ],
    );
    app = agents.register_args(app);
    app.get_matches()
  };
  let initial_cells = match matches
    .value_of(thud_ui_common::FLAG_INITIAL_BOARD)
    .map(|x| x.parse::<thud_ui_common::InitialBoard>())
  {
    None => thud_game::board::Cells::default(),
    Some(Ok(x)) => x.cells(),
    Some(Err(e)) => panic!("Bad initial board configuration: {}", e),
  };
  let logging_level = match matches
    .value_of(thud_ui_common::FLAG_LOG_LEVEL)
    .map(|x| x.parse::<log::LevelFilter>())
  {
    Some(Ok(x)) => x,
    Some(Err(_)) => panic!(
      "Bad logging level '{}'",
      matches.value_of(thud_ui_common::FLAG_LOG_LEVEL).unwrap()
    ),
    None => log::LevelFilter::Info,
  };

  // Set up logging.
  thud_ui_common::init::init_logger(logging_level);

  let mut agent1 = agents.get_player_1_from_arguments(&matches).unwrap();
  let mut agent2 = agents.get_player_2_from_arguments(&matches).unwrap();
  let mut state = thud_game::state::State::new(
    initial_cells,
    &thud_game::board::TRANSPOSITIONAL_EQUIVALENCE,
  );
  while !state.terminated() {
    println!("state: {:?}", state);
    let action = agent1.propose_action(&state).unwrap();
    println!("agent 1 proposes action: {:?}", action);
    state.do_action(&action);
    println!("state: {:?}", state);
    if state.terminated() {
      break;
    }
    let action = agent2.propose_action(&state).unwrap();
    println!("agent 2 proposes action: {:?}", action);
    state.do_action(&action);
  }
  println!("state has terminated: {:?}", state);
  println!(
    "final score: dwarfs {}, trolls {}",
    state.score(thud_game::Role::Dwarf),
    state.score(thud_game::Role::Troll)
  );

  // // Prompt for play.
  // loop {
  //   println!(
  //     "{:?} player's turn. Enter coordinate of piece to move.",
  //     human_role
  //   );
  //   let c = thud_ui_console::prompt_for_piece(state.cells(), human_role);
  //   let piece_actions: Vec<thud_game::Action> = state.position_actions(c).collect();
  //   if piece_actions.is_empty() {
  //     println!("Piece at {:?} has no actions.", c);
  //   } else {
  //     if let Some(action) = thud_ui_console::select_one(&piece_actions) {
  //       let mut moved_state = state.clone();
  //       moved_state.do_action(&action);
  //       println!(
  //         "After action, board: {}",
  //         thud_game::board::format_board(moved_state.cells())
  //       );
  //       println!("Is this okay?");
  //       match thud_ui_console::select_one(&["y", "n"]) {
  //         Some(&"y") => {
  //           state = moved_state;
  //           graph.retain_reachable_from(&[&state]);
  //           break;
  //         }
  //         _ => continue,
  //       }
  //     }
  //   }
  // }
}
