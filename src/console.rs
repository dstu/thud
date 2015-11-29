use std::io;
use std::io::Write;

use ::Board;
use ::BoardContent;
use ::Coordinate;
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
            print!("{}", glyph(Coordinate::new(row, col).map(|c| board[c])))
        }
        println!("");
    }
}

pub fn read_coordinate() -> Coordinate {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    loop {
        input.clear();
        print!("row? ");
        stdout.flush().ok().expect("could not flush stdout");
        stdin.read_line(&mut input).ok().expect("could not read from stdin");
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
        stdin.read_line(&mut input).ok().expect("could not read from stdin");
        let col: u8 = match input.trim().parse() {
            Ok(c) if c <= 14 => c,
            _ => {
                println!("bad col");
                continue
            },
        };
        match Coordinate::new(row, col) {
            None => {
                println!("coordinate out of playable range");
                continue
            },
            Some(c) => return c,
        }
    }
}
