use std::io;
use std::io::Write;

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

pub fn read_coordinate() -> GridCoordinate {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    loop {
        input.clear();
        print!("row? ");
        stdout.flush().ok().expect("could not flush stdout");
        assert!(stdin.read_line(&mut input).is_ok());
        let row: u8 = match input.trim().parse() {
            Ok(r) if r <= 14 => r,
            _ => {
                println!("bad row");
                continue
            },
        };
        input.clear();
        print!("col? ");
        stdout.flush().ok().expect("could not flush stdout");
        assert!(stdin.read_line(&mut input).is_ok());
        let col: u8 = match input.trim().parse() {
            Ok(c) if c <= 14 => c,
            _ => {
                println!("bad col");
                continue
            },
        };
        return GridCoordinate::new(row, col)
    }
}
