use std::io;
use std::io::Write;

use ::board::Cells;
use ::board::Content;
use ::board::Coordinate;
use ::board::Token;

pub fn glyph(b: Option<Content>) -> &'static str {
    match b {
        Some(Content::Occupied(Token::Stone)) => "O",
        Some(Content::Occupied(Token::Dwarf)) => "d",
        Some(Content::Occupied(Token::Troll)) => "T",
        Some(Content::Empty) => "_",
        None => ".",
    }
}

pub fn write_board(board: &Cells) {
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
