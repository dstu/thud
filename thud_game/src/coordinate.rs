#[cfg(test)] use quickcheck::{Arbitrary, Gen};
#[cfg(test)] use rand::Rng;
#[cfg(test)] use rand::seq::SliceRandom;

use std::fmt;

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
  pub const fn new(row: u8, col: u8) -> Option<Self> {
    COORDINATE_LOOKUP[row as usize][col as usize]
  }

  pub const fn from_index(index: usize) -> Self {
    ALL_COORDINATES[index]
  }

  pub const fn new_unchecked(row: u8, col: u8) -> Self {
    Coordinate {
      packed: col * COL_MULTIPLIER + row * ROW_MULTIPLIER,
      index: compute_index(row, col),
    }
  }

  pub /* const */ fn all() -> &'static [Self] {
    ALL_COORDINATES
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

  pub fn index(&self) -> usize {
    self.index as usize
  }
}

impl fmt::Debug for Coordinate {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "({}, {})", self.row(), self.col())
  }
}

#[macro_export]
macro_rules! coordinate_literal {
  (0, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(0, 5)
  }; // 1
  (0, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(0, 6)
  }; // 2
  (0, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(0, 7)
  }; // 3
  (0, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(0, 8)
  }; // 4
  (0, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(0, 9)
  }; // 5

  (1, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 4)
  }; // 1
  (1, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 5)
  }; // 2
  (1, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 6)
  }; // 3
  (1, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 7)
  }; // 4
  (1, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 8)
  }; // 5
  (1, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 9)
  }; // 6
  (1, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(1, 10)
  }; // 7

  (2, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 3)
  }; // 1
  (2, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 4)
  }; // 2
  (2, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 5)
  }; // 3
  (2, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 6)
  }; // 4
  (2, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 7)
  }; // 5
  (2, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 8)
  }; // 6
  (2, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 9)
  }; // 7
  (2, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 10)
  }; // 8
  (2, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(2, 11)
  }; // 9

  (3, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 2)
  }; // 1
  (3, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 3)
  }; // 2
  (3, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 4)
  }; // 3
  (3, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 5)
  }; // 4
  (3, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 6)
  }; // 5
  (3, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 7)
  }; // 6
  (3, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 8)
  }; // 7
  (3, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 9)
  }; // 8
  (3, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 10)
  }; // 9
  (3, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 11)
  }; // 10
  (3, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(3, 12)
  }; // 11

  (4, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 1)
  }; // 1
  (4, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 2)
  }; // 2
  (4, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 3)
  }; // 3
  (4, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 4)
  }; // 4
  (4, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 5)
  }; // 5
  (4, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 6)
  }; // 6
  (4, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 7)
  }; // 7
  (4, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 8)
  }; // 8
  (4, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 9)
  }; // 9
  (4, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 10)
  }; // 10
  (4, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 11)
  }; // 11
  (4, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 12)
  }; // 12
  (4, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(4, 13)
  }; // 13

  (5, 0) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 0)
  }; // 1
  (5, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 1)
  }; // 2
  (5, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 2)
  }; // 3
  (5, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 3)
  }; // 4
  (5, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 4)
  }; // 5
  (5, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 5)
  }; // 6
  (5, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 6)
  }; // 7
  (5, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 7)
  }; // 8
  (5, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 8)
  }; // 9
  (5, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 9)
  }; // 10
  (5, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 10)
  }; // 11
  (5, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 11)
  }; // 12
  (5, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 12)
  }; // 13
  (5, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 13)
  }; // 14
  (5, 14) => {
    $crate::coordinate::Coordinate::new_unchecked(5, 14)
  }; // 15

  (6, 0) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 0)
  }; // 1
  (6, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 1)
  }; // 2
  (6, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 2)
  }; // 3
  (6, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 3)
  }; // 4
  (6, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 4)
  }; // 5
  (6, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 5)
  }; // 6
  (6, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 6)
  }; // 7
  (6, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 7)
  }; // 8
  (6, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 8)
  }; // 9
  (6, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 9)
  }; // 10
  (6, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 10)
  }; // 11
  (6, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 11)
  }; // 12
  (6, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 12)
  }; // 13
  (6, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 13)
  }; // 14
  (6, 14) => {
    $crate::coordinate::Coordinate::new_unchecked(6, 14)
  }; // 15

  (7, 0) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 0)
  }; // 1
  (7, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 1)
  }; // 2
  (7, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 2)
  }; // 3
  (7, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 3)
  }; // 4
  (7, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 4)
  }; // 5
  (7, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 5)
  }; // 6
  (7, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 6)
  }; // 7
  (7, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 7)
  }; // 8
  (7, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 8)
  }; // 9
  (7, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 9)
  }; // 10
  (7, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 10)
  }; // 11
  (7, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 11)
  }; // 12
  (7, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 12)
  }; // 13
  (7, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 13)
  }; // 14
  (7, 14) => {
    $crate::coordinate::Coordinate::new_unchecked(7, 14)
  }; // 15

  (8, 0) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 0)
  }; // 1
  (8, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 1)
  }; // 2
  (8, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 2)
  }; // 3
  (8, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 3)
  }; // 4
  (8, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 4)
  }; // 5
  (8, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 5)
  }; // 6
  (8, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 6)
  }; // 7
  (8, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 7)
  }; // 8
  (8, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 8)
  }; // 9
  (8, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 9)
  }; // 10
  (8, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 10)
  }; // 11
  (8, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 11)
  }; // 12
  (8, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 12)
  }; // 13
  (8, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 13)
  }; // 14
  (8, 14) => {
    $crate::coordinate::Coordinate::new_unchecked(8, 14)
  }; // 15

  (9, 0) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 0)
  }; // 1
  (9, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 1)
  }; // 2
  (9, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 2)
  }; // 3
  (9, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 3)
  }; // 4
  (9, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 4)
  }; // 5
  (9, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 5)
  }; // 6
  (9, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 6)
  }; // 7
  (9, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 7)
  }; // 8
  (9, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 8)
  }; // 9
  (9, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 9)
  }; // 10
  (9, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 10)
  }; // 11
  (9, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 11)
  }; // 12
  (9, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 12)
  }; // 13
  (9, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 13)
  }; // 14
  (9, 14) => {
    $crate::coordinate::Coordinate::new_unchecked(9, 14)
  }; // 15

  (10, 1) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 1)
  }; // 1
  (10, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 2)
  }; // 2
  (10, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 3)
  }; // 3
  (10, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 4)
  }; // 4
  (10, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 5)
  }; // 5
  (10, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 6)
  }; // 6
  (10, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 7)
  }; // 7
  (10, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 8)
  }; // 8
  (10, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 9)
  }; // 9
  (10, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 10)
  }; // 10
  (10, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 11)
  }; // 11
  (10, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 12)
  }; // 12
  (10, 13) => {
    $crate::coordinate::Coordinate::new_unchecked(10, 13)
  }; // 13

  (11, 2) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 2)
  }; // 1
  (11, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 3)
  }; // 2
  (11, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 4)
  }; // 3
  (11, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 5)
  }; // 4
  (11, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 6)
  }; // 5
  (11, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 7)
  }; // 6
  (11, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 8)
  }; // 7
  (11, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 9)
  }; // 8
  (11, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 10)
  }; // 9
  (11, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 11)
  }; // 10
  (11, 12) => {
    $crate::coordinate::Coordinate::new_unchecked(11, 12)
  }; // 11

  (12, 3) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 3)
  }; // 1
  (12, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 4)
  }; // 2
  (12, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 5)
  }; // 3
  (12, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 6)
  }; // 4
  (12, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 7)
  }; // 5
  (12, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 8)
  }; // 6
  (12, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 9)
  }; // 7
  (12, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 10)
  }; // 8
  (12, 11) => {
    $crate::coordinate::Coordinate::new_unchecked(12, 11)
  }; // 9

  (13, 4) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 4)
  }; // 1
  (13, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 5)
  }; // 2
  (13, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 6)
  }; // 3
  (13, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 7)
  }; // 4
  (13, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 8)
  }; // 5
  (13, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 9)
  }; // 6
  (13, 10) => {
    $crate::coordinate::Coordinate::new_unchecked(13, 10)
  }; // 7

  (14, 5) => {
    $crate::coordinate::Coordinate::new_unchecked(14, 5)
  }; // 1
  (14, 6) => {
    $crate::coordinate::Coordinate::new_unchecked(14, 6)
  }; // 2
  (14, 7) => {
    $crate::coordinate::Coordinate::new_unchecked(14, 7)
  }; // 3
  (14, 8) => {
    $crate::coordinate::Coordinate::new_unchecked(14, 8)
  }; // 4
  (14, 9) => {
    $crate::coordinate::Coordinate::new_unchecked(14, 9)
  }; // 5
}

const COORDINATE_LOOKUP: &'static [[Option<Coordinate>; 15]; 15] = &[
  [
    None,
    None,
    None,
    None,
    None,
    Some(coordinate_literal!(0, 5)),
    Some(coordinate_literal!(0, 6)),
    Some(coordinate_literal!(0, 7)),
    Some(coordinate_literal!(0, 8)),
    Some(coordinate_literal!(0, 9)),
    None,
    None,
    None,
    None,
    None,
  ],
  [
    None,
    None,
    None,
    None,
    Some(coordinate_literal!(1, 4)),
    Some(coordinate_literal!(1, 5)),
    Some(coordinate_literal!(1, 6)),
    Some(coordinate_literal!(1, 7)),
    Some(coordinate_literal!(1, 8)),
    Some(coordinate_literal!(1, 9)),
    Some(coordinate_literal!(1, 10)),
    None,
    None,
    None,
    None,
  ],
  [
    None,
    None,
    None,
    Some(coordinate_literal!(2, 3)),
    Some(coordinate_literal!(2, 4)),
    Some(coordinate_literal!(2, 5)),
    Some(coordinate_literal!(2, 6)),
    Some(coordinate_literal!(2, 7)),
    Some(coordinate_literal!(2, 8)),
    Some(coordinate_literal!(2, 9)),
    Some(coordinate_literal!(2, 10)),
    Some(coordinate_literal!(2, 11)),
    None,
    None,
    None,
  ],
  [
    None,
    None,
    Some(coordinate_literal!(3, 2)),
    Some(coordinate_literal!(3, 3)),
    Some(coordinate_literal!(3, 4)),
    Some(coordinate_literal!(3, 5)),
    Some(coordinate_literal!(3, 6)),
    Some(coordinate_literal!(3, 7)),
    Some(coordinate_literal!(3, 8)),
    Some(coordinate_literal!(3, 9)),
    Some(coordinate_literal!(3, 10)),
    Some(coordinate_literal!(3, 11)),
    Some(coordinate_literal!(3, 12)),
    None,
    None,
  ],
  [
    None,
    Some(coordinate_literal!(4, 1)),
    Some(coordinate_literal!(4, 2)),
    Some(coordinate_literal!(4, 3)),
    Some(coordinate_literal!(4, 4)),
    Some(coordinate_literal!(4, 5)),
    Some(coordinate_literal!(4, 6)),
    Some(coordinate_literal!(4, 7)),
    Some(coordinate_literal!(4, 8)),
    Some(coordinate_literal!(4, 9)),
    Some(coordinate_literal!(4, 10)),
    Some(coordinate_literal!(4, 11)),
    Some(coordinate_literal!(4, 12)),
    Some(coordinate_literal!(4, 13)),
    None,
  ],
  [
    Some(coordinate_literal!(5, 0)),
    Some(coordinate_literal!(5, 1)),
    Some(coordinate_literal!(5, 2)),
    Some(coordinate_literal!(5, 3)),
    Some(coordinate_literal!(5, 4)),
    Some(coordinate_literal!(5, 5)),
    Some(coordinate_literal!(5, 6)),
    Some(coordinate_literal!(5, 7)),
    Some(coordinate_literal!(5, 8)),
    Some(coordinate_literal!(5, 9)),
    Some(coordinate_literal!(5, 10)),
    Some(coordinate_literal!(5, 11)),
    Some(coordinate_literal!(5, 12)),
    Some(coordinate_literal!(5, 13)),
    Some(coordinate_literal!(5, 14)),
  ],
  [
    Some(coordinate_literal!(6, 0)),
    Some(coordinate_literal!(6, 1)),
    Some(coordinate_literal!(6, 2)),
    Some(coordinate_literal!(6, 3)),
    Some(coordinate_literal!(6, 4)),
    Some(coordinate_literal!(6, 5)),
    Some(coordinate_literal!(6, 6)),
    Some(coordinate_literal!(6, 7)),
    Some(coordinate_literal!(6, 8)),
    Some(coordinate_literal!(6, 9)),
    Some(coordinate_literal!(6, 10)),
    Some(coordinate_literal!(6, 11)),
    Some(coordinate_literal!(6, 12)),
    Some(coordinate_literal!(6, 13)),
    Some(coordinate_literal!(6, 14)),
  ],
  [
    Some(coordinate_literal!(7, 0)),
    Some(coordinate_literal!(7, 1)),
    Some(coordinate_literal!(7, 2)),
    Some(coordinate_literal!(7, 3)),
    Some(coordinate_literal!(7, 4)),
    Some(coordinate_literal!(7, 5)),
    Some(coordinate_literal!(7, 6)),
    Some(coordinate_literal!(7, 7)),
    Some(coordinate_literal!(7, 8)),
    Some(coordinate_literal!(7, 9)),
    Some(coordinate_literal!(7, 10)),
    Some(coordinate_literal!(7, 11)),
    Some(coordinate_literal!(7, 12)),
    Some(coordinate_literal!(7, 13)),
    Some(coordinate_literal!(7, 14)),
  ],
  [
    Some(coordinate_literal!(8, 0)),
    Some(coordinate_literal!(8, 1)),
    Some(coordinate_literal!(8, 2)),
    Some(coordinate_literal!(8, 3)),
    Some(coordinate_literal!(8, 4)),
    Some(coordinate_literal!(8, 5)),
    Some(coordinate_literal!(8, 6)),
    Some(coordinate_literal!(8, 7)),
    Some(coordinate_literal!(8, 8)),
    Some(coordinate_literal!(8, 9)),
    Some(coordinate_literal!(8, 10)),
    Some(coordinate_literal!(8, 11)),
    Some(coordinate_literal!(8, 12)),
    Some(coordinate_literal!(8, 13)),
    Some(coordinate_literal!(8, 14)),
  ],
  [
    Some(coordinate_literal!(9, 0)),
    Some(coordinate_literal!(9, 1)),
    Some(coordinate_literal!(9, 2)),
    Some(coordinate_literal!(9, 3)),
    Some(coordinate_literal!(9, 4)),
    Some(coordinate_literal!(9, 5)),
    Some(coordinate_literal!(9, 6)),
    Some(coordinate_literal!(9, 7)),
    Some(coordinate_literal!(9, 8)),
    Some(coordinate_literal!(9, 9)),
    Some(coordinate_literal!(9, 10)),
    Some(coordinate_literal!(9, 11)),
    Some(coordinate_literal!(9, 12)),
    Some(coordinate_literal!(9, 13)),
    Some(coordinate_literal!(9, 14)),
  ],
  [
    None,
    Some(coordinate_literal!(10, 1)),
    Some(coordinate_literal!(10, 2)),
    Some(coordinate_literal!(10, 3)),
    Some(coordinate_literal!(10, 4)),
    Some(coordinate_literal!(10, 5)),
    Some(coordinate_literal!(10, 6)),
    Some(coordinate_literal!(10, 7)),
    Some(coordinate_literal!(10, 8)),
    Some(coordinate_literal!(10, 9)),
    Some(coordinate_literal!(10, 10)),
    Some(coordinate_literal!(10, 11)),
    Some(coordinate_literal!(10, 12)),
    Some(coordinate_literal!(10, 13)),
    None,
  ],
  [
    None,
    None,
    Some(coordinate_literal!(11, 2)),
    Some(coordinate_literal!(11, 3)),
    Some(coordinate_literal!(11, 4)),
    Some(coordinate_literal!(11, 5)),
    Some(coordinate_literal!(11, 6)),
    Some(coordinate_literal!(11, 7)),
    Some(coordinate_literal!(11, 8)),
    Some(coordinate_literal!(11, 9)),
    Some(coordinate_literal!(11, 10)),
    Some(coordinate_literal!(11, 11)),
    Some(coordinate_literal!(11, 12)),
    None,
    None,
  ],
  [
    None,
    None,
    None,
    Some(coordinate_literal!(12, 3)),
    Some(coordinate_literal!(12, 4)),
    Some(coordinate_literal!(12, 5)),
    Some(coordinate_literal!(12, 6)),
    Some(coordinate_literal!(12, 7)),
    Some(coordinate_literal!(12, 8)),
    Some(coordinate_literal!(12, 9)),
    Some(coordinate_literal!(12, 10)),
    Some(coordinate_literal!(12, 11)),
    None,
    None,
    None,
  ],
  [
    None,
    None,
    None,
    None,
    Some(coordinate_literal!(13, 4)),
    Some(coordinate_literal!(13, 5)),
    Some(coordinate_literal!(13, 6)),
    Some(coordinate_literal!(13, 7)),
    Some(coordinate_literal!(13, 8)),
    Some(coordinate_literal!(13, 9)),
    Some(coordinate_literal!(13, 10)),
    None,
    None,
    None,
    None,
  ],
  [
    None,
    None,
    None,
    None,
    None,
    Some(coordinate_literal!(14, 5)),
    Some(coordinate_literal!(14, 6)),
    Some(coordinate_literal!(14, 7)),
    Some(coordinate_literal!(14, 8)),
    Some(coordinate_literal!(14, 9)),
    None,
    None,
    None,
    None,
    None,
  ],
];

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
  Some(Coordinate::new_unchecked(13, 9)),
];

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
  None,
];

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
  Some(Coordinate::new_unchecked(14, 8)),
];

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
  None,
];

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
  Some(Coordinate::new_unchecked(13, 8)),
];

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
  Some(Coordinate::new_unchecked(13, 10)),
];

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
  None,
];

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
  None,
];

const ALL_COORDINATES: &'static [Coordinate; 165] = &[
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
  Coordinate::new_unchecked(14, 9),
];

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
  (5u8, 9u8),
];

/// The offset in 1-d row-major order of `row`, which should be in [0, 14].
const ROW_OFFSET: &'static [u8; 15] = &[
  0, 5, 12, 21, 32, 45, 60, 75, 90, 105, 120, 133, 144, 153, 160,
];

/// A direction linking one Coordinate to another.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
  UpLeft,
  UpRight,
  DownLeft,
  DownRight,
}

const ALL_DIRECTIONS_REF: &'static [Direction] = &[
  Direction::Up,
  Direction::Down,
  Direction::Left,
  Direction::Right,
  Direction::UpLeft,
  Direction::UpRight,
  Direction::DownLeft,
  Direction::DownRight,
];

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

  pub /* const */ fn all() -> &'static [Direction] {
    ALL_DIRECTIONS_REF
  }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Convolution {
  i: u8,
}

impl Convolution {
  pub fn convolve(&self, c: Coordinate) -> Coordinate {
    match self.i {
      0 => c,
      1 => Coordinate::new_unchecked(14u8 - c.row(), c.col()),
      2 => Coordinate::new_unchecked(c.row(), 14u8 - c.col()),
      3 => Coordinate::new_unchecked(14u8 - c.row(), 14u8 - c.col()),
      4 => Coordinate::new_unchecked(c.col(), c.row()),
      5 => Coordinate::new_unchecked(14u8 - c.col(), c.row()),
      6 => Coordinate::new_unchecked(c.col(), 14u8 - c.row()),
      7 => Coordinate::new_unchecked(14u8 - c.col(), 14u8 - c.row()),
      _ => unreachable!(),
    }
  }

  pub fn inverse(&self, c: Coordinate) -> Coordinate {
    match self.i {
      0 => c,
      1 => Coordinate::new_unchecked(14u8 - c.row(), c.col()),
      2 => Coordinate::new_unchecked(c.row(), 14u8 - c.col()),
      3 => Coordinate::new_unchecked(14u8 - c.row(), 14u8 - c.col()),
      4 => Coordinate::new_unchecked(c.col(), c.row()),
      5 => Coordinate::new_unchecked(c.col(), 14u8 - c.row()),
      6 => Coordinate::new_unchecked(14u8 - c.col(), c.row()),
      7 => Coordinate::new_unchecked(14u8 - c.col(), 14u8 - c.row()),
      _ => unreachable!(),
    }
  }

  pub /* const */ fn all() -> &'static [Self] {
    ALL_CONVOLUTIONS
  }
}

const ALL_CONVOLUTIONS: &'static [Convolution; 8] = &[
  Convolution { i: 0 },
  Convolution { i: 1 },
  Convolution { i: 2 },
  Convolution { i: 3 },
  Convolution { i: 4 },
  Convolution { i: 5 },
  Convolution { i: 6 },
  Convolution { i: 7 },
];

// #[cfg(test)]
// impl Arbitrary for Coordinate {
//   fn arbitrary<G: Gen>(g: &mut G) -> Self {
//     *Coordinate::all().choose(g).unwrap()
//   }
// }

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoordinateLineSegment {
  pub start: Coordinate,
  pub direction: Direction,
  pub length: u8,
}

// #[cfg(test)]
// impl Arbitrary for Direction {
//   fn arbitrary<G: Gen>(g: &mut G) -> Self {
//     *Direction::all().choose(g).unwrap()
//   }
// }

// #[cfg(test)]
// impl Arbitrary for CoordinateLineSegment {
//   fn arbitrary<G: Gen>(g: &mut G) -> Self {
//     loop {
//       let start = Coordinate::arbitrary(g);
//       let direction = Direction::arbitrary(g);
//       if start.to_direction(direction).is_none() {
//         continue;
//       }

//       let mut length = 1u8;
//       let mut next = start.to_direction(direction);
//       loop {
//         match next {
//           None => break,
//           Some(n) => {
//             if g.gen_ratio(1, length as u32) {
//               break;
//             }
//             next = n.to_direction(direction);
//             length += 1;
//           }
//         }
//       }
//       return CoordinateLineSegment {
//         start: start,
//         direction: direction,
//         length: length,
//       };
//     }
//   }
// }

#[cfg(test)]
mod test {
  use super::{Convolution, Coordinate};

  #[test]
  fn convolved_inverse_ok() {
    for c in Coordinate::all() {
      for v in Convolution::all() {
        assert_eq!(*c, v.inverse(v.convolve(*c)));
      }
    }
  }
}
