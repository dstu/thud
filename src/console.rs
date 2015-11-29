use ::Board;
use ::BoardContent;
use ::GridCoordinate;
use ::Token;

pub fn glyph(b: Option<BoardContent>) -> &'static str {
    match b {
        Some(BoardContent::Occupied(Token::Stone)) => "O",
        Some(BoardContent::Occupied(Token::Dwarf)) => "d",
        Some(BoardContent::Occupied(Token::Troll)) => "T",
        Some(BoardContent::Empty) => "_",
        None => ".",
    }
}

pub fn write_board(board: &Board) {
    for row in 0u8..15u8 {
        for col in 0u8..15u8 {
            print!("{}", glyph(board.get_grid_square(GridCoordinate::new(row, col))))
        }
        println!("");
    }
}
