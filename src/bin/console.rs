extern crate fern;
extern crate log;
extern crate thud;
extern crate chrono;

use thud::game;
use thud::game::board;
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
    let mut state = game::State::new(
        board, String::from_str("Player 1").ok().expect(""), String::from_str("Player 2").ok().expect(""));
    let mut i = 0u8;
    while i < 2 {
        {
            let active = state.active_player();
            println!("Moves for {} ({:?}):", active.name(), active.role());
            for a in state.role_actions(active.role()) {
                println!("  {:?}", a);
            }
        }
        state.toggle_active_player();
        i += 1;
    }
}
