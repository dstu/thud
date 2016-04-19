extern crate fern;
extern crate log;
extern crate chrono;
extern crate mcts;
extern crate thud;
#[macro_use(coordinate_literal)] extern crate thud_game;

use thud::console_ui;
use thud_game::board;
use mcts::base::State;

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
    let mut board = board::Cells::new();
    board[coordinate_literal!(7, 6)] = board::Content::Occupied(board::Token::Troll);
    board[coordinate_literal!(8, 6)] = board::Content::Occupied(board::Token::Troll);
    board[coordinate_literal!(9, 6)] = board::Content::Occupied(board::Token::Troll);
    board[coordinate_literal!(3, 7)] = board::Content::Occupied(board::Token::Dwarf);
    board[coordinate_literal!(2, 7)] = board::Content::Occupied(board::Token::Dwarf);
    board[coordinate_literal!(1, 7)] = board::Content::Occupied(board::Token::Dwarf);
    board[coordinate_literal!(0, 7)] = board::Content::Occupied(board::Token::Dwarf);
    console_ui::write_board(&board);
    let mut state = State::new(board);
    let mut i = 0u8;
    while i < 2 {
        {
            println!("Moves for {:?}:", state.active_role());
            for a in state.role_actions(state.active_role()) {
                println!("  {:?}", a);
            }
        }
        state.toggle_active_role();
        i += 1;
    }
}
