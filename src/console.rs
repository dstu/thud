use ::BoardState;
use ::Coordinate;
use ::Piece;

pub fn glyph(p: Option<Piece>) -> &'static str {
    match p {
        Some(Piece::Thudstone) => "O",
        Some(Piece::Dwarf) => "d",
        Some(Piece::Troll) => "T",
        None => ".",
    }
}

pub fn write_board(board: &BoardState) {
    print!["     "];
    for i in 0u8..5u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["    "];
    for i in 5u8..12u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["   "];
    for i in 12u8..21u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["  "];
    for i in 21u8..32u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print![" "];
    for i in 32u8..45u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    for i in 45u8..60u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    for i in 60u8..75u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    for i in 75u8..90u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    for i in 90u8..105u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    for i in 105u8..120u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print![" "];
    for i in 120u8..133u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["  "];
    for i in 133u8..144u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["   "];
    for i in 144u8..153u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["    "];
    for i in 153u8..160u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
    print!["     "];
    for i in 160u8..165u8 {
        print!["{}", glyph(board.piece_at(Coordinate::new(i)))];
    }
    println![""];
}
