extern crate fern;
extern crate log;
extern crate thud;
extern crate chrono;

use thud::game::board;
use thud::mcts::State;
use std::str::FromStr;

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
    board[board::Coordinate::new(7, 6).unwrap()] = board::Content::Occupied(board::Token::Troll);
    board[board::Coordinate::new(8, 6).unwrap()] = board::Content::Occupied(board::Token::Troll);
    board[board::Coordinate::new(9, 6).unwrap()] = board::Content::Occupied(board::Token::Troll);
    board[board::Coordinate::new(3, 7).unwrap()] = board::Content::Occupied(board::Token::Dwarf);
    board[board::Coordinate::new(2, 7).unwrap()] = board::Content::Occupied(board::Token::Dwarf);
    board[board::Coordinate::new(1, 7).unwrap()] = board::Content::Occupied(board::Token::Dwarf);
    board[board::Coordinate::new(0, 7).unwrap()] = board::Content::Occupied(board::Token::Dwarf);
    thud::console_ui::write_board(&board);
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
