use super::Role;
use super::actions::{Action, ActionIterator,
                     DwarfCoordinateConsumer, DwarfDirectionConsumer,
                     TrollCoordinateConsumer, TrollDirectionConsumer};
use super::coordinate::{Coordinate, Direction};

use std::clone::Clone;
use std::default::Default;
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

    pub fn role_actions<'s>(&'s self, r: Role, allow_end_proposal: bool) -> ActionIterator<'s> {
        let occupied_cells = self.occupied_iter(r);
        match r {
            Role::Dwarf =>
                ActionIterator::for_dwarf(
                    allow_end_proposal,
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
                    allow_end_proposal,
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
            &Action::ProposeEnd => (),
            &Action::HandleEndProposal(_) => (),
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

    pub fn occupied_iter<'s>(&'s self, r: Role) -> OccupiedCellsIter<'s> {
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

    fn index(&self, c: Coordinate) -> &Content {
        &self.cells[c.index()]
    }
}

impl IndexMut<Coordinate> for Cells {
    fn index_mut(&mut self, c: Coordinate) -> &mut Content {
        &mut self.cells[c.index()]
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
        for c in Coordinate::all() {
            for i in 0..8 {
                board[c.convolved(i)].hash(&mut hashers[i as usize]);
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
    use ::coordinate::Coordinate;
    use super::{Cells, decode_board};

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
