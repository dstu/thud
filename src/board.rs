use ::actions::{Action, ActionIterator,
                DwarfCoordinateConsumer, DwarfDirectionConsumer,
                TrollCoordinateConsumer, TrollDirectionConsumer};
use ::Role;

use std::cmp::PartialEq;
use std::fmt;
use std::hash::{Hash, Hasher, SipHasher};
use std::ops::{Index, IndexMut};

/// A physical token on the game board.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    Stone,
    Dwarf,
    Troll,
}

impl Token {
    /// Returns the token's role, or `None` if the token is the Thudstone.
    pub fn role(&self) -> Option<Role> {
        match *self {
            Token::Stone => None,
            Token::Dwarf => Some(Role::Dwarf),
            Token::Troll => Some(Role::Troll),
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

    pub fn role(&self) -> Option<Role> {
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
/// column)`. The row and column should be in `[0, 15]`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Coordinate {
    packed: u8,
}

impl Coordinate {
    /// If `(row, col)` is a playable space on the gameboard, returns a
    /// coordinate referring to that space, otherwise `None`.
    pub fn new(row: u8, col: u8) -> Option<Self> {
        if row >= 15 || col >= 15 {
            return None
        }
        let (row_start, row_end) = bounds_of_row(row);
        if row_start <= col && col <= row_end {
            Some(Coordinate::new_unchecked(row, col))
        } else {
            None
        }
    }

    fn new_unchecked(row: u8, col: u8) -> Self {
        Coordinate { packed: col * COL_MULTIPLIER + row * ROW_MULTIPLIER, }
    }

    fn from_index(index: usize) -> Self {
        assert!(index < 165);
        let row = row_of_index(index);
        let row_offset = offset_of_row(row);
        let (row_start, _) = bounds_of_row(row);
        let col = (index - (row_offset as usize) + (row_start as usize)) as u8;
        Coordinate { packed: col * COL_MULTIPLIER + row * ROW_MULTIPLIER, }
    }

    pub fn row(&self) -> u8 {
        (self.packed & ROW_MASK) / ROW_MULTIPLIER
    }

    pub fn col(&self) -> u8 {
        (self.packed & COL_MASK) / COL_MULTIPLIER
    }

    pub fn at_leftmost(&self) -> bool {
        let (row_start, _) = bounds_of_row(self.row());
        self.col() == row_start
    }

    pub fn at_rightmost(&self) -> bool {
        let (_, row_end) = bounds_of_row(self.row());
        self.col() == row_end
    }

    pub fn at_top(&self) -> bool {
        match self.row() {
            0 => true,
            r @ 1...5 => {
                let col = self.col();
                let (row_start, row_end) = bounds_of_row(r);
                col == row_start || col == row_end
            },
            _ => false,
        }
    }

    pub fn at_bottom(&self) -> bool {
        match self.row() {
            r @ 9...13 => {
                let col = self.col();
                let (row_start, row_end) = bounds_of_row(r);
                col == row_start || col == row_end
            },
            14 => true,
            _ => false,
        }
    }

    pub fn to_left(&self) -> Option<Self> {
        if self.at_leftmost() {
            None
        } else {
            Some(Coordinate { packed: self.packed - COL_MULTIPLIER })
        }
    }

    pub fn to_right(&self) -> Option<Self> {
        if self.at_rightmost() {
            None
        } else {
            Some(Coordinate { packed: self.packed + COL_MULTIPLIER })
        }
    }

    pub fn to_up(&self) -> Option<Self> {
        if self.at_top() {
            None
        } else {
            Some(Coordinate { packed: self.packed - ROW_MULTIPLIER })
        }
    }

    pub fn to_down(&self) -> Option<Self> {
        if self.at_bottom() {
            None
        } else {
            Some(Coordinate { packed: self.packed + ROW_MULTIPLIER })
        }
    }

   
    pub fn to_direction(&self, d: Direction) -> Option<Coordinate> {
        match d {
            Direction::Up => self.to_up(),
            Direction::UpLeft => self.to_up().and_then(|s| s.to_left()),
            Direction::UpRight => self.to_up().and_then(|s| s.to_right()),
            Direction::Down => self.to_down(),
            Direction::DownLeft => self.to_down().and_then(|s| s.to_left()),
            Direction::DownRight => self.to_down().and_then(|s| s.to_right()),
            Direction::Left => self.to_left(),
            Direction::Right => self.to_right(),
        }
    }

    fn index(&self) -> usize {
        let row_offset = offset_of_row(self.row());
        let (row_start, _) = bounds_of_row(self.row());
        (row_offset + self.col() - row_start) as usize
    }
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.row(), self.col())
    }
}

/// Returns the start and end columns of `row`, which must be in [0, 14].
fn bounds_of_row(row: u8) -> (u8, u8) {
    let row_start = match row {
        r @ 0...4 => 5 - r,
        r @ 10...14 => r - 9,
        _ => 0,
    };
    let row_end = 14 - row_start;
    (row_start, row_end)
}

/// Returns the offset in 1-d row-major order of `row`, which should be in [0, 14].
fn offset_of_row(row: u8) -> u8 {
    match row {
        0 => 0,
        1 => 5,
        2 => 12,
        3 => 21,
        4 => 32,
        5 => 45,
        6 => 60,
        7 => 75,
        8 => 90,
        9 => 105,
        10 => 120,
        11 => 133,
        12 => 144,
        13 => 153,
        14 => 160,
        _ => unreachable!(),
    }
}

/// Returns the row (in 15x15 gridspace) that `index` (in linear cellspace) is
/// in.
fn row_of_index(index: usize) -> u8 {
    match index {
        0...4 => 0u8,
        5...11 => 1u8,
        12...20 => 2u8,
        21...31 => 3u8,
        31...44 => 4u8,
        45...59 => 5u8,
        60...74 => 6u8,
        75...89 => 7u8,
        90...104 => 8u8,
        105...119 => 9u8,
        120...132 => 10u8,
        133...143 => 11u8,
        144...152 => 12u8,
        153...159 => 13u8,
        160...164 => 14u8,
        _ => unreachable!(),
    }
}

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

/// Describes a line on the game board proceeding in some direction.
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
pub struct Cells {
    cells: [Content; 165],
}

impl Cells {
    /// Creates an empty board.
    pub fn new() -> Self {
        Cells { cells: [Content::Empty; 165], }
    }

    /// Creates a standard starting board.
    pub fn default() -> Self {
        let mut b = Cells::new();

        b[Coordinate::new(6, 6).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(6, 7).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(6, 8).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(7, 6).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(7, 7).unwrap()] = Content::Occupied(Token::Stone);
        b[Coordinate::new(7, 8).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(8, 6).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(8, 7).unwrap()] = Content::Occupied(Token::Troll);
        b[Coordinate::new(8, 8).unwrap()] = Content::Occupied(Token::Troll);

        b[Coordinate::new(0, 5).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 6).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 8).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 9).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(1, 10).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(2, 11).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(3, 12).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(4, 13).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(5, 0).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(6, 0).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(8, 0).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(9, 0).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(10, 13).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(11, 12).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(12, 11).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(13, 10).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(14, 5).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 6).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 8).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 9).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(13, 4).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(12, 3).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(11, 2).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(10, 1).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(5, 14).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(6, 14).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(8, 14).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(9, 14).unwrap()] = Content::Occupied(Token::Dwarf);

        b[Coordinate::new(4, 1).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(3, 2).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(2, 3).unwrap()] = Content::Occupied(Token::Dwarf);
        b[Coordinate::new(1, 4).unwrap()] = Content::Occupied(Token::Dwarf);

        b
    }

    pub fn role_actions<'s>(&'s self, r: Role) -> ActionIterator<'s> {
        let occupied_cells = self.occupied_iter(r);
        match r {
            Role::Dwarf =>
                ActionIterator::for_dwarf(
                    occupied_cells.flat_map(DwarfCoordinateConsumer::new(self))),
                //  The above provides a concrete type for these iterator transforms:
                //  occupied_cells.flat_map(|position| {
                //         Direction::all()
                //             .into_iter()
                //             .flat_map(|d| (MoveIterator::new(self, position, *d)
                //                            .chain(HurlIterator::new(self, position, *d))))
                // })),
            Role::Troll =>
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
            Content::Occupied(t) if t.role() == Some(Role::Dwarf) => {
                ActionIterator::for_dwarf_position(
                    Direction::all()
                        .into_iter()
                        .flat_map(DwarfDirectionConsumer::new(self, position)))
            },
            Content::Occupied(t) if t.role() == Some(Role::Troll) => {
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
            &Action::Shove(start, end, ref captured) => {
                self[end] = self[start];
                self[start] = Content::Empty;
                for c in captured {
                    self[*c] = Content::Empty;
                }
            },
            &Action::Concede => (),
        }
    }

    pub fn cells_iter<'s>(&'s self) -> ContentsIter<'s> {
        ContentsIter { board: self, index: 0, }
    }

    pub fn occupied_iter<'s>(&'s self, r: Role) -> OccupiedCellsIter<'s> {
        OccupiedCellsIter { board: self, role: r, index: 0, }
    }
}

impl Index<Coordinate> for Cells {
    type Output = Content;

    fn index(&self, i: Coordinate) -> &Content {
        &self.cells[i.index()]
    }
}

impl IndexMut<Coordinate> for Cells {
    fn index_mut(&mut self, i: Coordinate) -> &mut Content {
        &mut self.cells[i.index()]
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
            let coordinate = Coordinate::from_index(self.index);
            self.index += 1;
            Some((coordinate, self.board[coordinate]))
        }
    }
}

pub struct OccupiedCellsIter<'a> {
    board: &'a Cells,
    role: Role,
    index: usize,
}

impl<'a> Iterator for OccupiedCellsIter<'a> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        loop {
            if self.index >= self.board.cells.len() {
                return None
            } else {
                let coordinate = Coordinate::from_index(self.index);
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

pub struct Transpositions {
    board: Cells,
}

impl Transpositions {
    pub fn new(board: Cells) -> Self {
        Transpositions { board: board, }
    }
}

impl PartialEq<Cells> for Transpositions {
    fn eq(&self, other: &Cells) -> bool {
        for row in 0u8..8u8 {
            for col in 0u8..8u8 {
                for &c in [Coordinate::new_unchecked(row, col),
                          Coordinate::new_unchecked(7u8 - row, col),
                          Coordinate::new_unchecked(row, 7u8 - col),
                          Coordinate::new_unchecked(7u8 - row, 7u8 - col),
                          Coordinate::new_unchecked(col, row),
                          Coordinate::new_unchecked(7u8 - col, row),
                          Coordinate::new_unchecked(col, 7u8 - row),
                          Coordinate::new_unchecked(7u8 - col, 7u8 - row)].iter() {
                    if self.board[c] != other[c] {
                        return false
                    }
                }
            }
        }
        true
    }
}

impl Hash for Transpositions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hashers = [SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new()];
        for row in 0u8..8u8 {
            for col in 0u8..8u8 {
                let mut i = 0;
                for &c in &[Coordinate::new_unchecked(row, col),
                            Coordinate::new_unchecked(7u8 - row, col),
                            Coordinate::new_unchecked(row, 7u8 - col),
                            Coordinate::new_unchecked(7u8 - row, 7u8 - col),
                            Coordinate::new_unchecked(col, row),
                            Coordinate::new_unchecked(7u8 - col, row),
                            Coordinate::new_unchecked(col, 7u8 - row),
                            Coordinate::new_unchecked(7u8 - col, 7u8 - row)] {
                    self.board[c].hash(&mut hashers[i]);
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
                               hashers[7].finish()];
        (&mut hash_values).sort();
        for v in &hash_values {
            state.write_u64(*v);
        }
    }
}
