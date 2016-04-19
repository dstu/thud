use ::game::actions::{Action, ActionIterator,
                      DwarfCoordinateConsumer, DwarfDirectionConsumer,
                      TrollCoordinateConsumer, TrollDirectionConsumer};
use ::game;

use std::clone::Clone;
use std::default::Default;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::hash::{Hash, Hasher, SipHasher};

/// A physical token on the game board.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    Stone,
    Dwarf,
    Troll,
}

impl Token {
    /// Returns the token's role, or `None` if the token is the Thudstone.
    pub fn role(&self) -> Option<game::Role> {
        match *self {
            Token::Stone => None,
            Token::Dwarf => Some(game::Role::Dwarf),
            Token::Troll => Some(game::Role::Troll),
        }
    }
}

/// The content of a space on the board.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Content {
    Occupied(Token),
    Empty,
}

impl Content {
    pub fn is_empty(&self) -> bool {
        match *self {
            Content::Empty => true,
            _ => false,
        }
    }

    pub fn is_occupied(&self) -> bool {
        return !self.is_empty()
    }

    pub fn is_troll(&self) -> bool {
        *self == Content::Occupied(Token::Troll)
    }

    pub fn is_dwarf(&self) -> bool {
        *self == Content::Occupied(Token::Dwarf)
    }

    pub fn role(&self) -> Option<game::Role> {
        match *self {
            Content::Empty => None,
            Content::Occupied(t) => t.role(),
        }
    }
}

const COL_MULTIPLIER: u8 = 0x10u8;
const COL_MASK: u8 = 0xF0u8;
const ROW_MULTIPLIER: u8 = 1u8;
const ROW_MASK: u8 = 0x0Fu8;

/// A space on the game board where a piece may be placed.
///
/// Coordinates are created by providing a pair of values: `Coordinate::new(row,
/// column)`. The row and column should be in `[0, 15)`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Coordinate {
    packed: u8,
    index: u8,
}

const fn compute_index(row: u8, col: u8) -> u8 {
    ROW_OFFSET[row as usize] + col - ROW_BOUNDS[row as usize].0
}

impl Coordinate {
    /// If `(row, col)` is a playable space on the gameboard, returns a
    /// coordinate referring to that space, otherwise `None`.
    pub fn new(row: u8, col: u8) -> Option<Self> {
        if row >= 15 || col >= 15 {
            return None
        }
        let (row_start, row_end) = ROW_BOUNDS[row as usize];
        if row_start <= col && col <= row_end {
            Some(Coordinate::new_unchecked(row, col))
        } else {
            None
        }
    }

    pub const fn new_unchecked(row: u8, col: u8) -> Self {
        Coordinate { packed: col * COL_MULTIPLIER + row * ROW_MULTIPLIER,
                     index: compute_index(row, col), }
    }

    pub fn convolved(&self, i: u8) -> Coordinate {
        match i {
            0 => *self,
            1 => Coordinate::new_unchecked(14u8 - self.row(), self.col()),
            2 => Coordinate::new_unchecked(self.row(), 14u8 - self.col()),
            3 => Coordinate::new_unchecked(14u8 - self.row(), 14u8 - self.col()),
            4 => Coordinate::new_unchecked(self.col(), self.row()),
            5 => Coordinate::new_unchecked(14u8 - self.col(), self.row()),
            6 => Coordinate::new_unchecked(self.col(), 14u8 - self.row()),
            7 => Coordinate::new_unchecked(14u8 - self.col(), 14u8 - self.row()),
            _ => panic!("Invalid convolution number {}", i),
        }
    }

    pub fn row(&self) -> u8 {
        (self.packed & ROW_MASK) / ROW_MULTIPLIER
    }

    pub fn col(&self) -> u8 {
        (self.packed & COL_MASK) / COL_MULTIPLIER
    }

    pub fn to_left(&self) -> Option<Self> {
        LEFT_NEIGHBORS[self.index as usize]
    }

    pub fn to_right(&self) -> Option<Self> {
        RIGHT_NEIGHBORS[self.index as usize]
    }

    pub fn to_up(&self) -> Option<Self> {
        UP_NEIGHBORS[self.index as usize]
    }

    pub fn to_up_left(&self) -> Option<Self> {
        UP_LEFT_NEIGHBORS[self.index as usize]
    }

    pub fn to_up_right(&self) -> Option<Self> {
        UP_RIGHT_NEIGHBORS[self.index as usize]
    }

    pub fn to_down(&self) -> Option<Self> {
        DOWN_NEIGHBORS[self.index as usize]
    }

    pub fn to_down_left(&self) -> Option<Self> {
        DOWN_LEFT_NEIGHBORS[self.index as usize]
    }

    pub fn to_down_right(&self) -> Option<Self> {
        DOWN_RIGHT_NEIGHBORS[self.index as usize]
    }
   
    pub fn to_direction(&self, d: Direction) -> Option<Coordinate> {
        match d {
            Direction::Left => self.to_left(),
            Direction::Right => self.to_right(),
            Direction::Up => self.to_up(),
            Direction::UpLeft => self.to_up_left(),
            Direction::UpRight => self.to_up_right(),
            Direction::Down => self.to_down(),
            Direction::DownLeft => self.to_down_left(),
            Direction::DownRight => self.to_down_right(),
        }
    }
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.row(), self.col())
    }
}

#[macro_export] macro_rules! coordinate_literal {
    (0, 5)   => ($crate::game::board::Coordinate::new_unchecked(0, 5));    // 1
    (0, 6)   => ($crate::game::board::Coordinate::new_unchecked(0, 6));    // 2
    (0, 7)   => ($crate::game::board::Coordinate::new_unchecked(0, 7));    // 3
    (0, 8)   => ($crate::game::board::Coordinate::new_unchecked(0, 8));    // 4
    (0, 9)   => ($crate::game::board::Coordinate::new_unchecked(0, 9));    // 5

    (1, 4)   => ($crate::game::board::Coordinate::new_unchecked(1, 4));    // 1
    (1, 5)   => ($crate::game::board::Coordinate::new_unchecked(1, 5));    // 2
    (1, 6)   => ($crate::game::board::Coordinate::new_unchecked(1, 6));    // 3
    (1, 7)   => ($crate::game::board::Coordinate::new_unchecked(1, 7));    // 4
    (1, 8)   => ($crate::game::board::Coordinate::new_unchecked(1, 8));    // 5
    (1, 9)   => ($crate::game::board::Coordinate::new_unchecked(1, 9));    // 6
    (1, 10)  => ($crate::game::board::Coordinate::new_unchecked(1, 10));   // 7

    (2, 3)   => ($crate::game::board::Coordinate::new_unchecked(2, 3));    // 1
    (2, 4)   => ($crate::game::board::Coordinate::new_unchecked(2, 4));    // 2
    (2, 5)   => ($crate::game::board::Coordinate::new_unchecked(2, 5));    // 3
    (2, 6)   => ($crate::game::board::Coordinate::new_unchecked(2, 6));    // 4
    (2, 7)   => ($crate::game::board::Coordinate::new_unchecked(2, 7));    // 5
    (2, 8)   => ($crate::game::board::Coordinate::new_unchecked(2, 8));    // 6
    (2, 9)   => ($crate::game::board::Coordinate::new_unchecked(2, 9));    // 7
    (2, 10)  => ($crate::game::board::Coordinate::new_unchecked(2, 10));   // 8
    (2, 11)  => ($crate::game::board::Coordinate::new_unchecked(2, 11));   // 9

    (3, 2)   => ($crate::game::board::Coordinate::new_unchecked(3, 2));    // 1
    (3, 3)   => ($crate::game::board::Coordinate::new_unchecked(3, 3));    // 2
    (3, 4)   => ($crate::game::board::Coordinate::new_unchecked(3, 4));    // 3
    (3, 5)   => ($crate::game::board::Coordinate::new_unchecked(3, 5));    // 4
    (3, 6)   => ($crate::game::board::Coordinate::new_unchecked(3, 6));    // 5
    (3, 7)   => ($crate::game::board::Coordinate::new_unchecked(3, 7));    // 6
    (3, 8)   => ($crate::game::board::Coordinate::new_unchecked(3, 8));    // 7
    (3, 9)   => ($crate::game::board::Coordinate::new_unchecked(3, 9));    // 8
    (3, 10)  => ($crate::game::board::Coordinate::new_unchecked(3, 10));   // 9
    (3, 11)  => ($crate::game::board::Coordinate::new_unchecked(3, 11));   // 10
    (3, 12)  => ($crate::game::board::Coordinate::new_unchecked(3, 12));   // 11

    (4, 1)   => ($crate::game::board::Coordinate::new_unchecked(4, 1));    // 1
    (4, 2)   => ($crate::game::board::Coordinate::new_unchecked(4, 2));    // 2 
    (4, 3)   => ($crate::game::board::Coordinate::new_unchecked(4, 3));    // 3 
    (4, 4)   => ($crate::game::board::Coordinate::new_unchecked(4, 4));    // 4 
    (4, 5)   => ($crate::game::board::Coordinate::new_unchecked(4, 5));    // 5 
    (4, 6)   => ($crate::game::board::Coordinate::new_unchecked(4, 6));    // 6 
    (4, 7)   => ($crate::game::board::Coordinate::new_unchecked(4, 7));    // 7 
    (4, 8)   => ($crate::game::board::Coordinate::new_unchecked(4, 8));    // 8 
    (4, 9)   => ($crate::game::board::Coordinate::new_unchecked(4, 9));    // 9
    (4, 10)  => ($crate::game::board::Coordinate::new_unchecked(4, 10));   // 10
    (4, 11)  => ($crate::game::board::Coordinate::new_unchecked(4, 11));   // 11
    (4, 12)  => ($crate::game::board::Coordinate::new_unchecked(4, 12));   // 12
    (4, 13)  => ($crate::game::board::Coordinate::new_unchecked(4, 13));   // 13

    (5, 0)   => ($crate::game::board::Coordinate::new_unchecked(5, 0));    // 1 
    (5, 1)   => ($crate::game::board::Coordinate::new_unchecked(5, 1));    // 2 
    (5, 2)   => ($crate::game::board::Coordinate::new_unchecked(5, 2));    // 3 
    (5, 3)   => ($crate::game::board::Coordinate::new_unchecked(5, 3));    // 4 
    (5, 4)   => ($crate::game::board::Coordinate::new_unchecked(5, 4));    // 5 
    (5, 5)   => ($crate::game::board::Coordinate::new_unchecked(5, 5));    // 6 
    (5, 6)   => ($crate::game::board::Coordinate::new_unchecked(5, 6));    // 7 
    (5, 7)   => ($crate::game::board::Coordinate::new_unchecked(5, 7));    // 8 
    (5, 8)   => ($crate::game::board::Coordinate::new_unchecked(5, 8));    // 9 
    (5, 9)   => ($crate::game::board::Coordinate::new_unchecked(5, 9));    // 10
    (5, 10)  => ($crate::game::board::Coordinate::new_unchecked(5, 10));   // 11
    (5, 11)  => ($crate::game::board::Coordinate::new_unchecked(5, 11));   // 12
    (5, 12)  => ($crate::game::board::Coordinate::new_unchecked(5, 12));   // 13
    (5, 13)  => ($crate::game::board::Coordinate::new_unchecked(5, 13));   // 14
    (5, 14)  => ($crate::game::board::Coordinate::new_unchecked(5, 14));   // 15

    (6, 0)   => ($crate::game::board::Coordinate::new_unchecked(6, 0));    // 1 
    (6, 1)   => ($crate::game::board::Coordinate::new_unchecked(6, 1));    // 2 
    (6, 2)   => ($crate::game::board::Coordinate::new_unchecked(6, 2));    // 3 
    (6, 3)   => ($crate::game::board::Coordinate::new_unchecked(6, 3));    // 4 
    (6, 4)   => ($crate::game::board::Coordinate::new_unchecked(6, 4));    // 5 
    (6, 5)   => ($crate::game::board::Coordinate::new_unchecked(6, 5));    // 6 
    (6, 6)   => ($crate::game::board::Coordinate::new_unchecked(6, 6));    // 7 
    (6, 7)   => ($crate::game::board::Coordinate::new_unchecked(6, 7));    // 8 
    (6, 8)   => ($crate::game::board::Coordinate::new_unchecked(6, 8));    // 9 
    (6, 9)   => ($crate::game::board::Coordinate::new_unchecked(6, 9));    // 10
    (6, 10)  => ($crate::game::board::Coordinate::new_unchecked(6, 10));   // 11
    (6, 11)  => ($crate::game::board::Coordinate::new_unchecked(6, 11));   // 12
    (6, 12)  => ($crate::game::board::Coordinate::new_unchecked(6, 12));   // 13
    (6, 13)  => ($crate::game::board::Coordinate::new_unchecked(6, 13));   // 14
    (6, 14)  => ($crate::game::board::Coordinate::new_unchecked(6, 14));   // 15

    (7, 0)   => ($crate::game::board::Coordinate::new_unchecked(7, 0));    // 1 
    (7, 1)   => ($crate::game::board::Coordinate::new_unchecked(7, 1));    // 2 
    (7, 2)   => ($crate::game::board::Coordinate::new_unchecked(7, 2));    // 3 
    (7, 3)   => ($crate::game::board::Coordinate::new_unchecked(7, 3));    // 4 
    (7, 4)   => ($crate::game::board::Coordinate::new_unchecked(7, 4));    // 5 
    (7, 5)   => ($crate::game::board::Coordinate::new_unchecked(7, 5));    // 6 
    (7, 6)   => ($crate::game::board::Coordinate::new_unchecked(7, 6));    // 7 
    (7, 7)   => ($crate::game::board::Coordinate::new_unchecked(7, 7));    // 8 
    (7, 8)   => ($crate::game::board::Coordinate::new_unchecked(7, 8));    // 9 
    (7, 9)   => ($crate::game::board::Coordinate::new_unchecked(7, 9));    // 10
    (7, 10)  => ($crate::game::board::Coordinate::new_unchecked(7, 10));   // 11
    (7, 11)  => ($crate::game::board::Coordinate::new_unchecked(7, 11));   // 12
    (7, 12)  => ($crate::game::board::Coordinate::new_unchecked(7, 12));   // 13
    (7, 13)  => ($crate::game::board::Coordinate::new_unchecked(7, 13));   // 14
    (7, 14)  => ($crate::game::board::Coordinate::new_unchecked(7, 14));   // 15

    (8, 0)   => ($crate::game::board::Coordinate::new_unchecked(8, 0));    // 1 
    (8, 1)   => ($crate::game::board::Coordinate::new_unchecked(8, 1));    // 2 
    (8, 2)   => ($crate::game::board::Coordinate::new_unchecked(8, 2));    // 3 
    (8, 3)   => ($crate::game::board::Coordinate::new_unchecked(8, 3));    // 4 
    (8, 4)   => ($crate::game::board::Coordinate::new_unchecked(8, 4));    // 5 
    (8, 5)   => ($crate::game::board::Coordinate::new_unchecked(8, 5));    // 6 
    (8, 6)   => ($crate::game::board::Coordinate::new_unchecked(8, 6));    // 7 
    (8, 7)   => ($crate::game::board::Coordinate::new_unchecked(8, 7));    // 8 
    (8, 8)   => ($crate::game::board::Coordinate::new_unchecked(8, 8));    // 9 
    (8, 9)   => ($crate::game::board::Coordinate::new_unchecked(8, 9));    // 10
    (8, 10)  => ($crate::game::board::Coordinate::new_unchecked(8, 10));   // 11
    (8, 11)  => ($crate::game::board::Coordinate::new_unchecked(8, 11));   // 12
    (8, 12)  => ($crate::game::board::Coordinate::new_unchecked(8, 12));   // 13
    (8, 13)  => ($crate::game::board::Coordinate::new_unchecked(8, 13));   // 14
    (8, 14)  => ($crate::game::board::Coordinate::new_unchecked(8, 14));   // 15

    (9, 0)   => ($crate::game::board::Coordinate::new_unchecked(9, 0));    // 1 
    (9, 1)   => ($crate::game::board::Coordinate::new_unchecked(9, 1));    // 2 
    (9, 2)   => ($crate::game::board::Coordinate::new_unchecked(9, 2));    // 3 
    (9, 3)   => ($crate::game::board::Coordinate::new_unchecked(9, 3));    // 4 
    (9, 4)   => ($crate::game::board::Coordinate::new_unchecked(9, 4));    // 5 
    (9, 5)   => ($crate::game::board::Coordinate::new_unchecked(9, 5));    // 6 
    (9, 6)   => ($crate::game::board::Coordinate::new_unchecked(9, 6));    // 7 
    (9, 7)   => ($crate::game::board::Coordinate::new_unchecked(9, 7));    // 8 
    (9, 8)   => ($crate::game::board::Coordinate::new_unchecked(9, 8));    // 9 
    (9, 9)   => ($crate::game::board::Coordinate::new_unchecked(9, 9));    // 10
    (9, 10)  => ($crate::game::board::Coordinate::new_unchecked(9, 10));   // 11
    (9, 11)  => ($crate::game::board::Coordinate::new_unchecked(9, 11));   // 12
    (9, 12)  => ($crate::game::board::Coordinate::new_unchecked(9, 12));   // 13
    (9, 13)  => ($crate::game::board::Coordinate::new_unchecked(9, 13));   // 14
    (9, 14)  => ($crate::game::board::Coordinate::new_unchecked(9, 14));   // 15

    (10, 1)  => ($crate::game::board::Coordinate::new_unchecked(10, 1));   // 1
    (10, 2)  => ($crate::game::board::Coordinate::new_unchecked(10, 2));   // 2 
    (10, 3)  => ($crate::game::board::Coordinate::new_unchecked(10, 3));   // 3 
    (10, 4)  => ($crate::game::board::Coordinate::new_unchecked(10, 4));   // 4 
    (10, 5)  => ($crate::game::board::Coordinate::new_unchecked(10, 5));   // 5 
    (10, 6)  => ($crate::game::board::Coordinate::new_unchecked(10, 6));   // 6 
    (10, 7)  => ($crate::game::board::Coordinate::new_unchecked(10, 7));   // 7 
    (10, 8)  => ($crate::game::board::Coordinate::new_unchecked(10, 8));   // 8 
    (10, 9)  => ($crate::game::board::Coordinate::new_unchecked(10, 9));   // 9
    (10, 10) => ($crate::game::board::Coordinate::new_unchecked(10, 10));  // 10
    (10, 11) => ($crate::game::board::Coordinate::new_unchecked(10, 11));  // 11
    (10, 12) => ($crate::game::board::Coordinate::new_unchecked(10, 12));  // 12
    (10, 13) => ($crate::game::board::Coordinate::new_unchecked(10, 13));  // 13

    (11, 2)  => ($crate::game::board::Coordinate::new_unchecked(11, 2));   // 1
    (11, 3)  => ($crate::game::board::Coordinate::new_unchecked(11, 3));   // 2
    (11, 4)  => ($crate::game::board::Coordinate::new_unchecked(11, 4));   // 3
    (11, 5)  => ($crate::game::board::Coordinate::new_unchecked(11, 5));   // 4
    (11, 6)  => ($crate::game::board::Coordinate::new_unchecked(11, 6));   // 5
    (11, 7)  => ($crate::game::board::Coordinate::new_unchecked(11, 7));   // 6
    (11, 8)  => ($crate::game::board::Coordinate::new_unchecked(11, 8));   // 7
    (11, 9)  => ($crate::game::board::Coordinate::new_unchecked(11, 9));   // 8
    (11, 10) => ($crate::game::board::Coordinate::new_unchecked(11, 10));  // 9
    (11, 11) => ($crate::game::board::Coordinate::new_unchecked(11, 11));  // 10
    (11, 12) => ($crate::game::board::Coordinate::new_unchecked(11, 12));  // 11

    (12, 3)  => ($crate::game::board::Coordinate::new_unchecked(12, 3));   // 1
    (12, 4)  => ($crate::game::board::Coordinate::new_unchecked(12, 4));   // 2
    (12, 5)  => ($crate::game::board::Coordinate::new_unchecked(12, 5));   // 3
    (12, 6)  => ($crate::game::board::Coordinate::new_unchecked(12, 6));   // 4
    (12, 7)  => ($crate::game::board::Coordinate::new_unchecked(12, 7));   // 5
    (12, 8)  => ($crate::game::board::Coordinate::new_unchecked(12, 8));   // 6
    (12, 9)  => ($crate::game::board::Coordinate::new_unchecked(12, 9));   // 7
    (12, 10) => ($crate::game::board::Coordinate::new_unchecked(12, 10));  // 8
    (12, 11) => ($crate::game::board::Coordinate::new_unchecked(12, 11));  // 9

    (13, 4)  => ($crate::game::board::Coordinate::new_unchecked(13, 4));   // 1
    (13, 5)  => ($crate::game::board::Coordinate::new_unchecked(13, 5));   // 2
    (13, 6)  => ($crate::game::board::Coordinate::new_unchecked(13, 6));   // 3
    (13, 7)  => ($crate::game::board::Coordinate::new_unchecked(13, 7));   // 4
    (13, 8)  => ($crate::game::board::Coordinate::new_unchecked(13, 8));   // 5
    (13, 9)  => ($crate::game::board::Coordinate::new_unchecked(13, 9));   // 6
    (13, 10) => ($crate::game::board::Coordinate::new_unchecked(13, 10));  // 7

    (14, 5)  => ($crate::game::board::Coordinate::new_unchecked(14, 5));   // 1
    (14, 6)  => ($crate::game::board::Coordinate::new_unchecked(14, 6));   // 2
    (14, 7)  => ($crate::game::board::Coordinate::new_unchecked(14, 7));   // 3
    (14, 8)  => ($crate::game::board::Coordinate::new_unchecked(14, 8));   // 4
    (14, 9)  => ($crate::game::board::Coordinate::new_unchecked(14, 9));   // 5
}

static UP_NEIGHBORS: [Option<Coordinate>; 165] = [
    None,
    None,
    None,
    None,
    None,
    None,
    Some(Coordinate::new_unchecked(0, 5)),
    Some(Coordinate::new_unchecked(0, 6)),
    Some(Coordinate::new_unchecked(0, 7)),
    Some(Coordinate::new_unchecked(0, 8)),
    Some(Coordinate::new_unchecked(0, 9)),
    None,
    None,
    Some(Coordinate::new_unchecked(1, 4)),
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(1, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(2, 3)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(2, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(3, 2)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(3, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(4, 1)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(4, 13)),
    None,
    Some(Coordinate::new_unchecked(5, 0)),
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    Some(Coordinate::new_unchecked(5, 14)),
    Some(Coordinate::new_unchecked(6, 0)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    Some(Coordinate::new_unchecked(6, 14)),
    Some(Coordinate::new_unchecked(7, 0)),
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    Some(Coordinate::new_unchecked(7, 14)),
    Some(Coordinate::new_unchecked(8, 0)),
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(8, 14)),
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),];

static DOWN_NEIGHBORS: [Option<Coordinate>; 165] = [
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    Some(Coordinate::new_unchecked(6, 0)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    Some(Coordinate::new_unchecked(6, 14)),
    Some(Coordinate::new_unchecked(7, 0)),
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    Some(Coordinate::new_unchecked(7, 14)),
    Some(Coordinate::new_unchecked(8, 0)),
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(8, 14)),
    Some(Coordinate::new_unchecked(9, 0)),
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    Some(Coordinate::new_unchecked(9, 14)),
    None,
    Some(Coordinate::new_unchecked(10, 1)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(10, 13)),
    None,
    None,
    Some(Coordinate::new_unchecked(11, 2)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(11, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(12, 3)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(12, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(13, 4)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    Some(Coordinate::new_unchecked(13, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(14, 5)),
    Some(Coordinate::new_unchecked(14, 6)),
    Some(Coordinate::new_unchecked(14, 7)),
    Some(Coordinate::new_unchecked(14, 8)),
    Some(Coordinate::new_unchecked(14, 9)),
    None,
    None,
    None,
    None,
    None,
    None,];

static LEFT_NEIGHBORS: [Option<Coordinate>; 165] = [
    None,
    Some(Coordinate::new_unchecked(0, 5)),
    Some(Coordinate::new_unchecked(0, 6)),
    Some(Coordinate::new_unchecked(0, 7)),
    Some(Coordinate::new_unchecked(0, 8)),
    None,
    Some(Coordinate::new_unchecked(1, 4)),
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    None,
    Some(Coordinate::new_unchecked(2, 3)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    None,
    Some(Coordinate::new_unchecked(3, 2)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    None,
    Some(Coordinate::new_unchecked(4, 1)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    None,
    Some(Coordinate::new_unchecked(5, 0)),
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    None,
    Some(Coordinate::new_unchecked(6, 0)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    None,
    Some(Coordinate::new_unchecked(7, 0)),
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    None,
    Some(Coordinate::new_unchecked(8, 0)),
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    None,
    Some(Coordinate::new_unchecked(9, 0)),
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    None,
    Some(Coordinate::new_unchecked(10, 1)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    None,
    Some(Coordinate::new_unchecked(11, 2)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    None,
    Some(Coordinate::new_unchecked(12, 3)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    None,
    Some(Coordinate::new_unchecked(13, 4)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    None,
    Some(Coordinate::new_unchecked(14, 5)),
    Some(Coordinate::new_unchecked(14, 6)),
    Some(Coordinate::new_unchecked(14, 7)),
    Some(Coordinate::new_unchecked(14, 8)),];

static RIGHT_NEIGHBORS: [Option<Coordinate>; 165] = [
    Some(Coordinate::new_unchecked(0, 6)),
    Some(Coordinate::new_unchecked(0, 7)),
    Some(Coordinate::new_unchecked(0, 8)),
    Some(Coordinate::new_unchecked(0, 9)),
    None,
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(1, 10)),
    None,
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(2, 11)),
    None,
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(3, 12)),
    None,
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(4, 13)),
    None,
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    Some(Coordinate::new_unchecked(5, 14)),
    None,
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    Some(Coordinate::new_unchecked(6, 14)),
    None,
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    Some(Coordinate::new_unchecked(7, 14)),
    None,
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(8, 14)),
    None,
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    Some(Coordinate::new_unchecked(9, 14)),
    None,
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(10, 13)),
    None,
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(11, 12)),
    None,
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(12, 11)),
    None,
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    Some(Coordinate::new_unchecked(13, 10)),
    None,
    Some(Coordinate::new_unchecked(14, 6)),
    Some(Coordinate::new_unchecked(14, 7)),
    Some(Coordinate::new_unchecked(14, 8)),
    Some(Coordinate::new_unchecked(14, 9)),
    None,];

static UP_LEFT_NEIGHBORS: [Option<Coordinate>; 165] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(Coordinate::new_unchecked(0, 5)),
    Some(Coordinate::new_unchecked(0, 6)),
    Some(Coordinate::new_unchecked(0, 7)),
    Some(Coordinate::new_unchecked(0, 8)),
    Some(Coordinate::new_unchecked(0, 9)),
    None,
    None,
    Some(Coordinate::new_unchecked(1, 4)),
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(1, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(2, 3)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(2, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(3, 2)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(3, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(4, 1)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(4, 13)),
    None,
    Some(Coordinate::new_unchecked(5, 0)),
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    None,
    Some(Coordinate::new_unchecked(6, 0)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    None,
    Some(Coordinate::new_unchecked(7, 0)),
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    None,
    Some(Coordinate::new_unchecked(8, 0)),
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(9, 0)),
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(10, 1)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(11, 2)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(12, 3)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(13, 4)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),];

static UP_RIGHT_NEIGHBORS: [Option<Coordinate>; 165] = [
    None,
    None,
    None,
    None,
    None,
    Some(Coordinate::new_unchecked(0, 5)),
    Some(Coordinate::new_unchecked(0, 6)),
    Some(Coordinate::new_unchecked(0, 7)),
    Some(Coordinate::new_unchecked(0, 8)),
    Some(Coordinate::new_unchecked(0, 9)),
    None,
    None,
    Some(Coordinate::new_unchecked(1, 4)),
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(1, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(2, 3)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(2, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(3, 2)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(3, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(4, 1)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(4, 13)),
    None,
    None,
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    Some(Coordinate::new_unchecked(5, 14)),
    None,
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    Some(Coordinate::new_unchecked(6, 14)),
    None,
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    Some(Coordinate::new_unchecked(7, 14)),
    None,
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(8, 14)),
    None,
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    Some(Coordinate::new_unchecked(9, 14)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(10, 13)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(11, 12)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(12, 11)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    Some(Coordinate::new_unchecked(13, 10)),];

static DOWN_LEFT_NEIGHBORS: [Option<Coordinate>; 165] = [
    Some(Coordinate::new_unchecked(1, 4)),
    Some(Coordinate::new_unchecked(1, 5)),
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(2, 3)),
    Some(Coordinate::new_unchecked(2, 4)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(3, 2)),
    Some(Coordinate::new_unchecked(3, 3)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(4, 1)),
    Some(Coordinate::new_unchecked(4, 2)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(5, 0)),
    Some(Coordinate::new_unchecked(5, 1)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    None,
    Some(Coordinate::new_unchecked(6, 0)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    None,
    Some(Coordinate::new_unchecked(7, 0)),
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    None,
    Some(Coordinate::new_unchecked(8, 0)),
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    None,
    Some(Coordinate::new_unchecked(9, 0)),
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    None,
    None,
    Some(Coordinate::new_unchecked(10, 1)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(10, 13)),
    None,
    None,
    Some(Coordinate::new_unchecked(11, 2)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(11, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(12, 3)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(12, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(13, 4)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    Some(Coordinate::new_unchecked(13, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(14, 5)),
    Some(Coordinate::new_unchecked(14, 6)),
    Some(Coordinate::new_unchecked(14, 7)),
    Some(Coordinate::new_unchecked(14, 8)),
    Some(Coordinate::new_unchecked(14, 9)),
    None,
    None,
    None,
    None,
    None,];

static DOWN_RIGHT_NEIGHBORS: [Option<Coordinate>; 165] = [
    Some(Coordinate::new_unchecked(1, 6)),
    Some(Coordinate::new_unchecked(1, 7)),
    Some(Coordinate::new_unchecked(1, 8)),
    Some(Coordinate::new_unchecked(1, 9)),
    Some(Coordinate::new_unchecked(1, 10)),
    Some(Coordinate::new_unchecked(2, 5)),
    Some(Coordinate::new_unchecked(2, 6)),
    Some(Coordinate::new_unchecked(2, 7)),
    Some(Coordinate::new_unchecked(2, 8)),
    Some(Coordinate::new_unchecked(2, 9)),
    Some(Coordinate::new_unchecked(2, 10)),
    Some(Coordinate::new_unchecked(2, 11)),
    Some(Coordinate::new_unchecked(3, 4)),
    Some(Coordinate::new_unchecked(3, 5)),
    Some(Coordinate::new_unchecked(3, 6)),
    Some(Coordinate::new_unchecked(3, 7)),
    Some(Coordinate::new_unchecked(3, 8)),
    Some(Coordinate::new_unchecked(3, 9)),
    Some(Coordinate::new_unchecked(3, 10)),
    Some(Coordinate::new_unchecked(3, 11)),
    Some(Coordinate::new_unchecked(3, 12)),
    Some(Coordinate::new_unchecked(4, 3)),
    Some(Coordinate::new_unchecked(4, 4)),
    Some(Coordinate::new_unchecked(4, 5)),
    Some(Coordinate::new_unchecked(4, 6)),
    Some(Coordinate::new_unchecked(4, 7)),
    Some(Coordinate::new_unchecked(4, 8)),
    Some(Coordinate::new_unchecked(4, 9)),
    Some(Coordinate::new_unchecked(4, 10)),
    Some(Coordinate::new_unchecked(4, 11)),
    Some(Coordinate::new_unchecked(4, 12)),
    Some(Coordinate::new_unchecked(4, 13)),
    Some(Coordinate::new_unchecked(5, 2)),
    Some(Coordinate::new_unchecked(5, 3)),
    Some(Coordinate::new_unchecked(5, 4)),
    Some(Coordinate::new_unchecked(5, 5)),
    Some(Coordinate::new_unchecked(5, 6)),
    Some(Coordinate::new_unchecked(5, 7)),
    Some(Coordinate::new_unchecked(5, 8)),
    Some(Coordinate::new_unchecked(5, 9)),
    Some(Coordinate::new_unchecked(5, 10)),
    Some(Coordinate::new_unchecked(5, 11)),
    Some(Coordinate::new_unchecked(5, 12)),
    Some(Coordinate::new_unchecked(5, 13)),
    Some(Coordinate::new_unchecked(5, 14)),
    Some(Coordinate::new_unchecked(6, 1)),
    Some(Coordinate::new_unchecked(6, 2)),
    Some(Coordinate::new_unchecked(6, 3)),
    Some(Coordinate::new_unchecked(6, 4)),
    Some(Coordinate::new_unchecked(6, 5)),
    Some(Coordinate::new_unchecked(6, 6)),
    Some(Coordinate::new_unchecked(6, 7)),
    Some(Coordinate::new_unchecked(6, 8)),
    Some(Coordinate::new_unchecked(6, 9)),
    Some(Coordinate::new_unchecked(6, 10)),
    Some(Coordinate::new_unchecked(6, 11)),
    Some(Coordinate::new_unchecked(6, 12)),
    Some(Coordinate::new_unchecked(6, 13)),
    Some(Coordinate::new_unchecked(6, 14)),
    None,
    Some(Coordinate::new_unchecked(7, 1)),
    Some(Coordinate::new_unchecked(7, 2)),
    Some(Coordinate::new_unchecked(7, 3)),
    Some(Coordinate::new_unchecked(7, 4)),
    Some(Coordinate::new_unchecked(7, 5)),
    Some(Coordinate::new_unchecked(7, 6)),
    Some(Coordinate::new_unchecked(7, 7)),
    Some(Coordinate::new_unchecked(7, 8)),
    Some(Coordinate::new_unchecked(7, 9)),
    Some(Coordinate::new_unchecked(7, 10)),
    Some(Coordinate::new_unchecked(7, 11)),
    Some(Coordinate::new_unchecked(7, 12)),
    Some(Coordinate::new_unchecked(7, 13)),
    Some(Coordinate::new_unchecked(7, 14)),
    None,
    Some(Coordinate::new_unchecked(8, 1)),
    Some(Coordinate::new_unchecked(8, 2)),
    Some(Coordinate::new_unchecked(8, 3)),
    Some(Coordinate::new_unchecked(8, 4)),
    Some(Coordinate::new_unchecked(8, 5)),
    Some(Coordinate::new_unchecked(8, 6)),
    Some(Coordinate::new_unchecked(8, 7)),
    Some(Coordinate::new_unchecked(8, 8)),
    Some(Coordinate::new_unchecked(8, 9)),
    Some(Coordinate::new_unchecked(8, 10)),
    Some(Coordinate::new_unchecked(8, 11)),
    Some(Coordinate::new_unchecked(8, 12)),
    Some(Coordinate::new_unchecked(8, 13)),
    Some(Coordinate::new_unchecked(8, 14)),
    None,
    Some(Coordinate::new_unchecked(9, 1)),
    Some(Coordinate::new_unchecked(9, 2)),
    Some(Coordinate::new_unchecked(9, 3)),
    Some(Coordinate::new_unchecked(9, 4)),
    Some(Coordinate::new_unchecked(9, 5)),
    Some(Coordinate::new_unchecked(9, 6)),
    Some(Coordinate::new_unchecked(9, 7)),
    Some(Coordinate::new_unchecked(9, 8)),
    Some(Coordinate::new_unchecked(9, 9)),
    Some(Coordinate::new_unchecked(9, 10)),
    Some(Coordinate::new_unchecked(9, 11)),
    Some(Coordinate::new_unchecked(9, 12)),
    Some(Coordinate::new_unchecked(9, 13)),
    Some(Coordinate::new_unchecked(9, 14)),
    None,
    Some(Coordinate::new_unchecked(10, 1)),
    Some(Coordinate::new_unchecked(10, 2)),
    Some(Coordinate::new_unchecked(10, 3)),
    Some(Coordinate::new_unchecked(10, 4)),
    Some(Coordinate::new_unchecked(10, 5)),
    Some(Coordinate::new_unchecked(10, 6)),
    Some(Coordinate::new_unchecked(10, 7)),
    Some(Coordinate::new_unchecked(10, 8)),
    Some(Coordinate::new_unchecked(10, 9)),
    Some(Coordinate::new_unchecked(10, 10)),
    Some(Coordinate::new_unchecked(10, 11)),
    Some(Coordinate::new_unchecked(10, 12)),
    Some(Coordinate::new_unchecked(10, 13)),
    None,
    None,
    Some(Coordinate::new_unchecked(11, 2)),
    Some(Coordinate::new_unchecked(11, 3)),
    Some(Coordinate::new_unchecked(11, 4)),
    Some(Coordinate::new_unchecked(11, 5)),
    Some(Coordinate::new_unchecked(11, 6)),
    Some(Coordinate::new_unchecked(11, 7)),
    Some(Coordinate::new_unchecked(11, 8)),
    Some(Coordinate::new_unchecked(11, 9)),
    Some(Coordinate::new_unchecked(11, 10)),
    Some(Coordinate::new_unchecked(11, 11)),
    Some(Coordinate::new_unchecked(11, 12)),
    None,
    None,
    Some(Coordinate::new_unchecked(12, 3)),
    Some(Coordinate::new_unchecked(12, 4)),
    Some(Coordinate::new_unchecked(12, 5)),
    Some(Coordinate::new_unchecked(12, 6)),
    Some(Coordinate::new_unchecked(12, 7)),
    Some(Coordinate::new_unchecked(12, 8)),
    Some(Coordinate::new_unchecked(12, 9)),
    Some(Coordinate::new_unchecked(12, 10)),
    Some(Coordinate::new_unchecked(12, 11)),
    None,
    None,
    Some(Coordinate::new_unchecked(13, 4)),
    Some(Coordinate::new_unchecked(13, 5)),
    Some(Coordinate::new_unchecked(13, 6)),
    Some(Coordinate::new_unchecked(13, 7)),
    Some(Coordinate::new_unchecked(13, 8)),
    Some(Coordinate::new_unchecked(13, 9)),
    Some(Coordinate::new_unchecked(13, 10)),
    None,
    None,
    Some(Coordinate::new_unchecked(14, 5)),
    Some(Coordinate::new_unchecked(14, 6)),
    Some(Coordinate::new_unchecked(14, 7)),
    Some(Coordinate::new_unchecked(14, 8)),
    Some(Coordinate::new_unchecked(14, 9)),
    None,
    None,
    None,
    None,
    None,
    None,
    None,];

static ALL_COORDINATES: [Coordinate; 165] = [
    Coordinate::new_unchecked(0, 5),
    Coordinate::new_unchecked(0, 6),
    Coordinate::new_unchecked(0, 7),
    Coordinate::new_unchecked(0, 8),
    Coordinate::new_unchecked(0, 9),

    Coordinate::new_unchecked(1, 4),
    Coordinate::new_unchecked(1, 5),
    Coordinate::new_unchecked(1, 6),
    Coordinate::new_unchecked(1, 7),
    Coordinate::new_unchecked(1, 8),
    Coordinate::new_unchecked(1, 9),
    Coordinate::new_unchecked(1, 10),

    Coordinate::new_unchecked(2, 3),
    Coordinate::new_unchecked(2, 4),
    Coordinate::new_unchecked(2, 5),
    Coordinate::new_unchecked(2, 6),
    Coordinate::new_unchecked(2, 7),
    Coordinate::new_unchecked(2, 8),
    Coordinate::new_unchecked(2, 9),
    Coordinate::new_unchecked(2, 10),
    Coordinate::new_unchecked(2, 11),

    Coordinate::new_unchecked(3, 2),
    Coordinate::new_unchecked(3, 3),
    Coordinate::new_unchecked(3, 4),
    Coordinate::new_unchecked(3, 5),
    Coordinate::new_unchecked(3, 6),
    Coordinate::new_unchecked(3, 7),
    Coordinate::new_unchecked(3, 8),
    Coordinate::new_unchecked(3, 9),
    Coordinate::new_unchecked(3, 10),
    Coordinate::new_unchecked(3, 11),
    Coordinate::new_unchecked(3, 12),

    Coordinate::new_unchecked(4, 1),
    Coordinate::new_unchecked(4, 2),
    Coordinate::new_unchecked(4, 3),
    Coordinate::new_unchecked(4, 4),
    Coordinate::new_unchecked(4, 5),
    Coordinate::new_unchecked(4, 6),
    Coordinate::new_unchecked(4, 7),
    Coordinate::new_unchecked(4, 8),
    Coordinate::new_unchecked(4, 9),
    Coordinate::new_unchecked(4, 10),
    Coordinate::new_unchecked(4, 11),
    Coordinate::new_unchecked(4, 12),
    Coordinate::new_unchecked(4, 13),

    Coordinate::new_unchecked(5, 0),
    Coordinate::new_unchecked(5, 1),
    Coordinate::new_unchecked(5, 2),
    Coordinate::new_unchecked(5, 3),
    Coordinate::new_unchecked(5, 4),
    Coordinate::new_unchecked(5, 5),
    Coordinate::new_unchecked(5, 6),
    Coordinate::new_unchecked(5, 7),
    Coordinate::new_unchecked(5, 8),
    Coordinate::new_unchecked(5, 9),
    Coordinate::new_unchecked(5, 10),
    Coordinate::new_unchecked(5, 11),
    Coordinate::new_unchecked(5, 12),
    Coordinate::new_unchecked(5, 13),
    Coordinate::new_unchecked(5, 14),

    Coordinate::new_unchecked(6, 0),
    Coordinate::new_unchecked(6, 1),
    Coordinate::new_unchecked(6, 2),
    Coordinate::new_unchecked(6, 3),
    Coordinate::new_unchecked(6, 4),
    Coordinate::new_unchecked(6, 5),
    Coordinate::new_unchecked(6, 6),
    Coordinate::new_unchecked(6, 7),
    Coordinate::new_unchecked(6, 8),
    Coordinate::new_unchecked(6, 9),
    Coordinate::new_unchecked(6, 10),
    Coordinate::new_unchecked(6, 11),
    Coordinate::new_unchecked(6, 12),
    Coordinate::new_unchecked(6, 13),
    Coordinate::new_unchecked(6, 14),

    Coordinate::new_unchecked(7, 0),
    Coordinate::new_unchecked(7, 1),
    Coordinate::new_unchecked(7, 2),
    Coordinate::new_unchecked(7, 3),
    Coordinate::new_unchecked(7, 4),
    Coordinate::new_unchecked(7, 5),
    Coordinate::new_unchecked(7, 6),
    Coordinate::new_unchecked(7, 7),
    Coordinate::new_unchecked(7, 8),
    Coordinate::new_unchecked(7, 9),
    Coordinate::new_unchecked(7, 10),
    Coordinate::new_unchecked(7, 11),
    Coordinate::new_unchecked(7, 12),
    Coordinate::new_unchecked(7, 13),
    Coordinate::new_unchecked(7, 14),

    Coordinate::new_unchecked(8, 0),
    Coordinate::new_unchecked(8, 1),
    Coordinate::new_unchecked(8, 2),
    Coordinate::new_unchecked(8, 3),
    Coordinate::new_unchecked(8, 4),
    Coordinate::new_unchecked(8, 5),
    Coordinate::new_unchecked(8, 6),
    Coordinate::new_unchecked(8, 7),
    Coordinate::new_unchecked(8, 8),
    Coordinate::new_unchecked(8, 9),
    Coordinate::new_unchecked(8, 10),
    Coordinate::new_unchecked(8, 11),
    Coordinate::new_unchecked(8, 12),
    Coordinate::new_unchecked(8, 13),
    Coordinate::new_unchecked(8, 14),

    Coordinate::new_unchecked(9, 0),
    Coordinate::new_unchecked(9, 1),
    Coordinate::new_unchecked(9, 2),
    Coordinate::new_unchecked(9, 3),
    Coordinate::new_unchecked(9, 4),
    Coordinate::new_unchecked(9, 5),
    Coordinate::new_unchecked(9, 6),
    Coordinate::new_unchecked(9, 7),
    Coordinate::new_unchecked(9, 8),
    Coordinate::new_unchecked(9, 9),
    Coordinate::new_unchecked(9, 10),
    Coordinate::new_unchecked(9, 11),
    Coordinate::new_unchecked(9, 12),
    Coordinate::new_unchecked(9, 13),
    Coordinate::new_unchecked(9, 14),

    Coordinate::new_unchecked(10, 1),
    Coordinate::new_unchecked(10, 2),
    Coordinate::new_unchecked(10, 3),
    Coordinate::new_unchecked(10, 4),
    Coordinate::new_unchecked(10, 5),
    Coordinate::new_unchecked(10, 6),
    Coordinate::new_unchecked(10, 7),
    Coordinate::new_unchecked(10, 8),
    Coordinate::new_unchecked(10, 9),
    Coordinate::new_unchecked(10, 10),
    Coordinate::new_unchecked(10, 11),
    Coordinate::new_unchecked(10, 12),
    Coordinate::new_unchecked(10, 13),

    Coordinate::new_unchecked(11, 2),
    Coordinate::new_unchecked(11, 3),
    Coordinate::new_unchecked(11, 4),
    Coordinate::new_unchecked(11, 5),
    Coordinate::new_unchecked(11, 6),
    Coordinate::new_unchecked(11, 7),
    Coordinate::new_unchecked(11, 8),
    Coordinate::new_unchecked(11, 9),
    Coordinate::new_unchecked(11, 10),
    Coordinate::new_unchecked(11, 11),
    Coordinate::new_unchecked(11, 12),

    Coordinate::new_unchecked(12, 3),
    Coordinate::new_unchecked(12, 4),
    Coordinate::new_unchecked(12, 5),
    Coordinate::new_unchecked(12, 6),
    Coordinate::new_unchecked(12, 7),
    Coordinate::new_unchecked(12, 8),
    Coordinate::new_unchecked(12, 9),
    Coordinate::new_unchecked(12, 10),
    Coordinate::new_unchecked(12, 11),

    Coordinate::new_unchecked(13, 4),
    Coordinate::new_unchecked(13, 5),
    Coordinate::new_unchecked(13, 6),
    Coordinate::new_unchecked(13, 7),
    Coordinate::new_unchecked(13, 8),
    Coordinate::new_unchecked(13, 9),
    Coordinate::new_unchecked(13, 10),

    Coordinate::new_unchecked(14, 5),
    Coordinate::new_unchecked(14, 6),
    Coordinate::new_unchecked(14, 7),
    Coordinate::new_unchecked(14, 8),
    Coordinate::new_unchecked(14, 9),];

/// The start and end columns of `row`, which must be in [0, 14].
const ROW_BOUNDS: &'static [(u8, u8); 15] = &[
    (5u8, 9u8),
    (4u8, 10u8),
    (3u8, 11u8),
    (2u8, 12u8),
    (1u8, 13u8),
    (0u8, 14u8),
    (0u8, 14u8),
    (0u8, 14u8),
    (0u8, 14u8),
    (0u8, 14u8),
    (1u8, 13u8),
    (2u8, 12u8),
    (3u8, 11u8),
    (4u8, 10u8),
    (5u8, 9u8)];

/// The offset in 1-d row-major order of `row`, which should be in [0, 14].
const ROW_OFFSET: &'static [u8; 15] = &[
    0,
    5,
    12,
    21,
    32,
    45,
    60,
    75,
    90,
    105,
    120,
    133,
    144,
    153,
    160,];

/// A direction linking one Coordinate to another.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight,
}

const ALL_DIRECTIONS_REF: &'static [Direction] = &[Direction::Up,
                                                   Direction::Down,
                                                   Direction::Left,
                                                   Direction::Right,
                                                   Direction::UpLeft,
                                                   Direction::UpRight,
                                                   Direction::DownLeft,
                                                   Direction::DownRight];

impl Direction {
    pub fn reverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::UpLeft => Direction::DownRight,
            Direction::UpRight => Direction::DownLeft,
            Direction::Down => Direction::Up,
            Direction::DownLeft => Direction::UpRight,
            Direction::DownRight => Direction::UpLeft,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn all() -> &'static [Direction] {
        ALL_DIRECTIONS_REF
    }
}

/// Describes a line on the game board proceeding in some direction to the
/// bounds of the board.
pub struct Ray {
    here: Option<Coordinate>,
    direction: Direction,
}

impl Ray {
    pub fn new(c: Coordinate, d: Direction, ) -> Self {
        Ray { here: Some(c), direction: d, }
    }

    pub fn reverse(&self) -> Self {
        Ray { here: self.here, direction: self.direction.reverse(), }
    }
}

impl Iterator for Ray {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        let old_here = self.here;
        if let Some(h) = self.here {
            self.here = h.to_direction(self.direction);
        }
        old_here
    }
}

/// Layout of tokens on the game board, with handles addressing into it and
/// mutating it.
///
/// The octagonal playing area consists of a 15 by 15 square board from which a
/// triangle of 15 squares in each corner has been removed. The Thudstone is
/// placed on the centre square of the board, where it remains for the entire
/// game and may not be moved onto or through. The eight trolls are placed onto
/// the eight squares orthogonally and diagonally adjacent to the Thudstone and
/// the thirty-two dwarfs are placed so as to occupy all the perimeter spaces
/// except for the four in the same horizontal or vertical line as the
/// Thudstone. One player takes control of the dwarfs, the other controls the
/// trolls. The dwarfs move first.
///
/// This gives us 165 spaces, each of which may contain a piece.
pub struct Cells {
    cells: [Content; 165],
}

impl Cells {
    /// Creates an empty board.
    pub fn new() -> Self {
        Cells { cells: [Content::Empty; 165], }
    }

    pub fn role_actions<'s>(&'s self, r: game::Role) -> ActionIterator<'s> {
        let occupied_cells = self.occupied_iter(r);
        match r {
            game::Role::Dwarf =>
                ActionIterator::for_dwarf(
                    occupied_cells.flat_map(DwarfCoordinateConsumer::new(self))),
                //  The above provides a concrete type for these iterator transforms:
                //  occupied_cells.flat_map(|position| {
                //         Direction::all()
                //             .into_iter()
                //             .flat_map(|d| (MoveIterator::new(self, position, *d)
                //                            .chain(HurlIterator::new(self, position, *d))))
                // })),
            game::Role::Troll =>
                ActionIterator::for_troll(
                    occupied_cells.flat_map(TrollCoordinateConsumer::new(self))),
                    //  The above provides a concrete type for these iterator transforms:
                    // occupied_cells.flat_map(|position| {
                    //     Direction::all()
                    //         .into_iter()
                    //         .flat_map(|d| (MoveIterator::new(self, position, *d).take(1)
                    //                        .chain(ShoveIterator::new(self, position, *d))))
                    // })),
        }
    }

    pub fn position_actions<'s>(&'s self, position: Coordinate) -> ActionIterator<'s> {
        match self[position] {
            Content::Occupied(t) if t.role() == Some(game::Role::Dwarf) => {
                ActionIterator::for_dwarf_position(
                    Direction::all()
                        .into_iter()
                        .flat_map(DwarfDirectionConsumer::new(self, position)))
            },
            Content::Occupied(t) if t.role() == Some(game::Role::Troll) => {
                ActionIterator::for_troll_position(
                    Direction::all()
                        .into_iter()
                        .flat_map(TrollDirectionConsumer::new(self, position)))
            },
            _ => ActionIterator::empty(),
        }
    }

    pub fn do_action(&mut self, a: &Action) {
        match a {
            &Action::Move(start, end) => {
                self[end] = self[start];
                self[start] = Content::Empty;
            },
            &Action::Hurl(start, end) => {
                self[end] = self[start];
                self[start] = Content::Empty;
            },
            &Action::Shove(start, end, len, ref captured) => {
                self[end] = self[start];
                self[start] = Content::Empty;
                for i in 0..len {
                    self[captured[i as usize]] = Content::Empty;
                }
            },
            // &Action::Concede => (),
        }
    }

    pub fn cells_iter<'s>(&'s self) -> ContentsIter<'s> {
        ContentsIter { board: self, index: 0, }
    }

    pub fn occupied_iter<'s>(&'s self, r: game::Role) -> OccupiedCellsIter<'s> {
        OccupiedCellsIter { board: self, role: r, index: 0, }
    }
}

impl Default for Cells {
    /// Creates a standard starting board.
    fn default() -> Self {
        decode_board(
r#"
.....dd_dd.....
....d_____d....
...d_______d...
..d_________d..
.d___________d.
d_____________d
d_____TTT_____d
______TOT______
d_____TTT_____d
d_____________d
.d___________d.
..d_________d..
...d_______d...
....d_____d....
.....dd_dd.....
"#)
    }
}

impl Clone for Cells {
    fn clone(&self) -> Self {
        let mut other = Cells::new();
        other.cells.clone_from_slice(&self.cells);
        other
    }

    fn clone_from(&mut self, source: &Cells) {
        self.cells.clone_from_slice(&source.cells);
    }
}

impl Index<Coordinate> for Cells {
    type Output = Content;

    fn index(&self, i: Coordinate) -> &Content {
        &self.cells[i.index as usize]
    }
}

impl IndexMut<Coordinate> for Cells {
    fn index_mut(&mut self, i: Coordinate) -> &mut Content {
        &mut self.cells[i.index as usize]
    }
}

pub struct ContentsIter<'a> {
    board: &'a Cells,
    index: usize,
}

impl<'a> Iterator for ContentsIter<'a> {
    type Item = (Coordinate, Content);

    fn next(&mut self) -> Option<(Coordinate, Content)> {
        if self.index >= self.board.cells.len() {
            None
        } else {
            let coordinate = ALL_COORDINATES[self.index];
            self.index += 1;
            Some((coordinate, self.board[coordinate]))
        }
    }
}

pub struct OccupiedCellsIter<'a> {
    board: &'a Cells,
    role: game::Role,
    index: usize,
}

impl<'a> Iterator for OccupiedCellsIter<'a> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        loop {
            if self.index >= self.board.cells.len() {
                return None
            } else {
                let coordinate = ALL_COORDINATES[self.index];
                self.index += 1;
                match self.board[coordinate] {
                    Content::Occupied(t) if t.role() == Some(self.role) =>
                        return Some(coordinate),
                    _ => continue,
                }
            }
        }
    }
}

pub trait CellEquivalence {
    fn hash_board<H>(board: &Cells, state: &mut H) where H: Hasher;
    fn boards_equal(b1: &Cells, b2: &Cells) -> bool;
}

pub struct SimpleEquivalence;

impl CellEquivalence for SimpleEquivalence {
    fn hash_board<H>(board: &Cells, state: &mut H) where H: Hasher {
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = Coordinate::new(row, col) {
                    board[c].hash(state)
                }
            }
        }
    }

    fn boards_equal(b1: &Cells, b2: &Cells) -> bool {
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = Coordinate::new(row, col) {
                    if b1[c] != b2[c] {
                        return false
                    }
                }
            }
        }
        true
    }
}

pub struct TranspositionalEquivalence;

impl CellEquivalence for TranspositionalEquivalence {
    fn hash_board<H>(board: &Cells, state: &mut H) where H: Hasher {
        let mut hashers = [SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),];
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                let mut i = 0;
                for &c in [Coordinate::new(row, col),
                           Coordinate::new(14u8 - row, col),
                           Coordinate::new(row, 14u8 - col),
                           Coordinate::new(14u8 - row, 14u8 - col),
                           Coordinate::new(col, row),
                           Coordinate::new(14u8 - col, row),
                           Coordinate::new(col, 14u8 - row),
                           Coordinate::new(14u8 - col, 14u8 - row)].iter() {
                    if let Some(c) = c {
                        board[c].hash(&mut hashers[i]);
                    }
                    i += 1;
                }
            }
        }
        let mut hash_values = [hashers[0].finish(),
                               hashers[1].finish(),
                               hashers[2].finish(),
                               hashers[3].finish(),
                               hashers[4].finish(),
                               hashers[5].finish(),
                               hashers[6].finish(),
                               hashers[7].finish(),];
        hash_values.sort();
        for v in hash_values.into_iter() {
            state.write_u64(*v);
        }
    }

    fn boards_equal(b1: &Cells, b2: &Cells) -> bool {
        let mut equivalences = [true,
                                true,
                                true,
                                true,
                                true,
                                true,
                                true,
                                true,];
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c1) = Coordinate::new(row, col) {
                    let mut i = 0;
                    for &c2 in [Coordinate::new(row, col),
                                Coordinate::new(14u8 - row, col),
                                Coordinate::new(row, 14u8 - col),
                                Coordinate::new(14u8 - row, 14u8 - col),
                                Coordinate::new(col, row),
                                Coordinate::new(14u8 - col, row),
                                Coordinate::new(col, 14u8 - row),
                                Coordinate::new(14u8 - col, 14u8 - row)].iter() {
                        if let Some(c2) = c2 {
                            if b1[c1] != b2[c2] {
                                equivalences[i] = false;
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
        equivalences[0] || equivalences[1] || equivalences[2] || equivalences[3]
            || equivalences[4] || equivalences[5] || equivalences[6] || equivalences[7]
    }
}

pub fn decode_board(encoded: &str) -> Cells {
    assert_eq!(encoded.len(), 241);
    let mut chars = encoded.chars().skip(1);  // Skip leading newline.
    let mut board = Cells::new();
    for row in 0u8..15u8 {
        for col in 0u8..15u8 {
            let value = chars.next().unwrap();
            if let Some(c) = Coordinate::new(row, col) {
                board[c] = match value {
                    'T' => Content::Occupied(Token::Troll),
                    'd' => Content::Occupied(Token::Dwarf),
                    'O' => Content::Occupied(Token::Stone),
                    '_' => Content::Empty,
                    x @ _ => panic!("Unrecognized character '{}' for coordinate {:?}", x, c),
                }
            } else {
                assert!(value == '.', "expected '.' at ({}, {}) but got '{}'",
                        row, col, value);
            }
        }
        assert!(chars.next().unwrap() == '\n');
    }
    board
}

fn glyph(b: Option<Content>) -> char {
    match b {
        Some(Content::Occupied(Token::Stone)) => 'O',
        Some(Content::Occupied(Token::Dwarf)) => 'd',
        Some(Content::Occupied(Token::Troll)) => 'T',
        Some(Content::Empty) => '_',
        None => '.',
    }
}

pub fn format_board(board: &Cells) -> String {
    let mut s = String::with_capacity(241);
    s.push('\n');
    for row in 0u8..15u8 {
        for col in 0u8..15u8 {
            s.push(glyph(Coordinate::new(row, col).map(|c| board[c])));
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod test {
    use super::{Cells, Coordinate, decode_board};

    #[test]
    fn decode_board_ok() {
        let decoded = decode_board(
r#"
.....dd_dd.....
....d_____d....
...d_______d...
..d_________d..
.d___________d.
d_____________d
d_____TTT_____d
______TOT______
d_____TTT_____d
d_____________d
.d___________d.
..d_________d..
...d_______d...
....d_____d....
.....dd_dd.....
"#);
        let default = Cells::default();
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                if let Some(c) = Coordinate::new(row, col) {
                    assert_eq!(decoded[c], default[c]);
                }
            }
        }
    }
}
