use std::iter::Iterator;
use std::ops::Index;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Piece {
    Thudstone,
    Dwarf,
    Troll,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Coordinate(u8);

const ROW_LENGTHS: [u8; 15] = [5, 7, 9, 11, 13, 15, 15, 15, 15, 15, 13, 11, 9, 7, 5];
const ROW_OFFSETS: [u8; 15] = [0, 5, 12, 21, 32, 45, 60, 75, 90, 105, 120, 133, 144, 153, 160];

// For now, all numerical operations assume that we're on a standard Thud grid.
impl Coordinate {
    pub fn new(c: u8) -> Self {
        assert!(c <= 165u8);
        Coordinate(c)
    }

    pub fn offset(self) -> u8 {
        let Coordinate(c) = self;
        c
    }

    pub fn row(self) -> u8 {
        let Coordinate(c) = self;
        let mut i = 0u8;
        loop {
            if c < ROW_OFFSETS[i as usize] {
                return i
            }
            i += 1;
        }
    }

    pub fn column(self) -> u8 {
        let Coordinate(c) = self;
        c - ROW_OFFSETS[self.row() as usize]
    }

    pub fn left(self) -> Option<Self> {
        match self.column() {
            0 => None,
            _ => {
                let Coordinate(c) = self;
                Some(Coordinate::new(c - 1))
            }
        }
    }

    pub fn right(self) -> Option<Self> {
        if self.column() == ROW_LENGTHS[self.row() as usize] - 1 {
            None
        } else {
            let Coordinate(c) = self;
            Some(Coordinate::new(c + 1))
        }
    }

    pub fn up(self) -> Option<Self> {
        let row = self.row();
        if row == 0u8 {
            None
        } else {
            let col = self.column();
            let prev_row_end = ROW_OFFSETS[(row - 1) as usize] + ROW_LENGTHS[(row - 1) as usize];
            if prev_row_end < col {
                None
            } else {
                Some(Coordinate::new(ROW_OFFSETS[(row - 1) as usize] + col))
            }
        }
    }

    pub fn down(self) -> Option<Self> {
        let row = self.row();
        if row == 14u8 {
            None
        } else {
            let col = self.column();
            let next_row_end = ROW_OFFSETS[(row + 1) as usize] + ROW_LENGTHS[(row + 1) as usize];
            if next_row_end < col {
                None
            } else {
                Some(Coordinate::new(ROW_OFFSETS[(row + 1) as usize] + col))
            }
        }
    }

    /// Returns an iterator over the coordinates adjacent to `c`.
    pub fn adjacent_coordinates_iter(self) -> AdjacentCoordinateIter {
        AdjacentCoordinateIter {
            index: 0u8,
            coordinates: [self.up(), self.down(), self.left(), self.right()],
        }
    }
}

pub struct AdjacentCoordinateIter {
    index: u8,
    coordinates: [Option<Coordinate>; 4],
}

impl Iterator for AdjacentCoordinateIter {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        while self.index < 4u8 {
            self.index += 1u8;
            if self.coordinates[self.index as usize].is_some() {
                return self.coordinates[self.index as usize]
            }
        }
        None
    }
}

///     The octagonal playing area consists of a 15 by 15 square board from
///     which a triangle of 15 squares in each corner has been removed. The
///     Thudstone is placed on the centre square of the board, where it remains
///     for the entire game and may not be moved onto or through. The eight
///     trolls are placed onto the eight squares orthogonally and diagonally
///     adjacent to the Thudstone and the thirty-two dwarfs are placed so as to
///     occupy all the perimeter spaces except for the four in the same
///     horizontal or vertical line as the Thudstone. One player takes control
///     of the dwarfs, the other controls the trolls. The dwarfs move first.
///
/// This gives us 165 spaces, each of which may contain a piece.
pub struct BoardState {
    spaces: [Option<Piece>; 165],
}

impl BoardState {
    /// Returns a freshly constructed board for a standard game of Thud.
    pub fn init() -> Self {
        let mut board = BoardState { spaces: [None; 165], };
        // Place Dwarves.
        let mut c = Coordinate::new(0u8);
        while board.piece_at(c) != Some(Piece::Dwarf) {
            let row = c.row();
            let col = c.column();
            if row != 7 && !(col == 2 && (row == 0 || row == 14)) {
                board.set_piece_at(c, Some(Piece::Dwarf));
            }
            c = c.right().or_else(|| c.down()).or_else(|| c.left()).or_else(|| c.up()).unwrap();
        }
        // Place Thudstone.
        c = Coordinate::new(82);
        board.set_piece_at(c, Some(Piece::Thudstone));
        // Place Trolls.
        board.set_piece_at(c.up().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.down().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.left().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.right().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.up().and_then(|n| n.left()).unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.up().and_then(|n| n.right()).unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.down().and_then(|n| n.left()).unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.down().and_then(|n| n.right()).unwrap(), Some(Piece::Troll));
        board
    }

    /// Returns the piece at `c`.
    pub fn piece_at(&self, c: Coordinate) -> Option<Piece> {
        let Coordinate(offset) = c;
        self.spaces[offset as usize]
    }

    /// Sets the piece at `c`.
    pub fn set_piece_at(&mut self, c: Coordinate, p: Option<Piece>) {
        let Coordinate(offset) = c;
        self.spaces[offset as usize] = p;
    }
}
