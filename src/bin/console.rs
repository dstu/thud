extern crate thud;

use std::str::FromStr;

fn main() {
    let mut board = thud::Board::new();
    board[thud::Coordinate::new(7, 6).unwrap()] = thud::BoardContent::Occupied(thud::Token::Troll);
    board[thud::Coordinate::new(8, 6).unwrap()] = thud::BoardContent::Occupied(thud::Token::Troll);
    board[thud::Coordinate::new(9, 6).unwrap()] = thud::BoardContent::Occupied(thud::Token::Troll);
    board[thud::Coordinate::new(3, 7).unwrap()] = thud::BoardContent::Occupied(thud::Token::Dwarf);
    board[thud::Coordinate::new(2, 7).unwrap()] = thud::BoardContent::Occupied(thud::Token::Dwarf);
    board[thud::Coordinate::new(1, 7).unwrap()] = thud::BoardContent::Occupied(thud::Token::Dwarf);
    board[thud::Coordinate::new(0, 7).unwrap()] = thud::BoardContent::Occupied(thud::Token::Dwarf);
    thud::console_ui::write_board(&board);
    let mut state = thud::GameState::new(
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
        state.toggle_player();
        i += 1;
    }
}
