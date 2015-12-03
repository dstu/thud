pub mod console;

use std::iter::Iterator;
use std::ops::{Index, IndexMut};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    Stone,
    Dwarf,
    Troll,
}

impl Token {
    pub fn role(&self) -> Option<Role> {
        match *self {
            Token::Stone => None,
            Token::Dwarf => Some(Role::Dwarf),
            Token::Troll => Some(Role::Troll),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoardContent {
    Occupied(Token),
    Empty,
}

impl BoardContent {
    pub fn is_empty(&self) -> bool {
        match *self {
            BoardContent::Empty => true,
            _ => false,
        }
    }

    pub fn is_occupied(&self) -> bool {
        return !self.is_empty()
    }

    pub fn is_troll(&self) -> bool {
        *self == BoardContent::Occupied(Token::Troll)
    }

    pub fn is_dwarf(&self) -> bool {
        *self == BoardContent::Occupied(Token::Dwarf)
    }

    pub fn role(&self) -> Option<Role> {
        match *self {
            BoardContent::Empty => None,
            BoardContent::Occupied(t) => t.role(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct BoardElement {
    pub content: BoardContent,
    pub coordinate: Coordinate,
}

const COL_MULTIPLIER: u8 = 0x10u8;
const COL_MASK: u8 = 0xF0u8;
const ROW_MULTIPLIER: u8 = 1u8;
const ROW_MASK: u8 = 0x0Fu8;

/// A space in the game, where a piece may be placed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Coordinate {
    packed: u8,
}

impl Coordinate {
    pub fn new(row: u8, col: u8) -> Option<Self> {
        if row >= 15 || col >= 15 {
            return None
        }
        let (row_start, row_end) = bounds_of_row(row);
        if row_start <= col && col <= row_end {
            Some(Coordinate { packed: col * COL_MULTIPLIER + row * ROW_MULTIPLIER, })
        } else {
            None
        }
    }

    fn from_index(index: usize) -> Self {
        // println!("from_index({})", index);
        assert!(index < 165);
        let row = row_of_index(index);
        let row_offset = offset_of_row(row);
        let (row_start, _) = bounds_of_row(row);
        let col = (index - (row_offset as usize) + (row_start as usize)) as u8;
        // println!("from_index({}) = ({}, {})", index, row, col);
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
        // println!("index of ({}, {}) = {} + {} - {}",
        //          self.row(), self.col(),
        //          row_offset, self.col(), row_start);
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

const ALL_DIRECTIONS: [Direction; 8] = [Direction::Up,
                                        Direction::Down,
                                        Direction::Left,
                                        Direction::Right,
                                        Direction::UpLeft,
                                        Direction::UpRight,
                                        Direction::DownLeft,
                                        Direction::DownRight];

const ALL_DIRECTIONS_REF: &'static [Direction] = &ALL_DIRECTIONS;

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
struct Ray {
    here: Option<Coordinate>,
    direction: Direction,
}

impl Ray {
    fn new(c: Coordinate, d: Direction, ) -> Self {
        Ray { here: Some(c), direction: d, }
    }

    fn reverse(&self) -> Self {
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
pub struct Board {
    cells: [BoardContent; 165],
}

impl Board {
    /// Creates an empty board.
    pub fn new() -> Self {
        Board { cells: [BoardContent::Empty; 165], }
    }

    /// Creates a standard starting board.
    pub fn default() -> Self {
        let mut b = Board::new();

        b[Coordinate::new(6, 6).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(6, 7).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(6, 8).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(7, 6).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(7, 7).unwrap()] = BoardContent::Occupied(Token::Stone);
        b[Coordinate::new(7, 8).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(8, 6).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(8, 7).unwrap()] = BoardContent::Occupied(Token::Troll);
        b[Coordinate::new(8, 8).unwrap()] = BoardContent::Occupied(Token::Troll);

        b[Coordinate::new(0, 5).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 6).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 8).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(0, 9).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(1, 10).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(2, 11).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(3, 12).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(4, 13).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(5, 0).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(6, 0).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(8, 0).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(9, 0).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(10, 13).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(11, 12).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(12, 11).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(13, 10).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(14, 5).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 6).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 8).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(14, 9).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(13, 4).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(12, 3).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(11, 2).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(10, 1).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(5, 14).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(6, 14).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(8, 14).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(9, 14).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b[Coordinate::new(4, 1).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(3, 2).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(2, 3).unwrap()] = BoardContent::Occupied(Token::Dwarf);
        b[Coordinate::new(1, 4).unwrap()] = BoardContent::Occupied(Token::Dwarf);

        b
    }

    pub fn actions(&self, r: Role) // -> ActionIterator 
    {
        for (index, content) in self.cells.into_iter().enumerate() {
            match *content {
                BoardContent::Occupied(t) if t.role() == Some(r) => {
                    let position = Coordinate::from_index(index);
                    println!("moves for {:?} @ {:?}", t, position);
                    match r {
                        Role::Dwarf => {
                            for d in Direction::all() {
                                // println!("moves going {:?}", d);
                                for a in MoveIterator::new(self, position, *d) {
                                    println!("  {:?}", a);
                                }
                                // println!("hurls going {:?}", d);
                                for a in HurlIterator::new(self, position, *d) {
                                    println!("  {:?}", a);
                                }
                            }
                        },
                        Role::Troll => {
                            for d in Direction::all() {
                                // println!("moves going {:?}", d);
                                for a in MoveIterator::new(self, position, *d).take(1) {
                                    println!("  {:?}", a);
                                }
                                // println!("shoves going {:?}", d);
                                for a in ShoveIterator::new(self, position, *d) {
                                    println!("  {:?}", a);
                                }
                            }
                        },
                    }
                },
                _ => continue,
            }
        }
    }
}

impl Index<Coordinate> for Board {
    type Output = BoardContent;

    fn index(&self, i: Coordinate) -> &BoardContent {
        &self.cells[i.index()]
    }
}

impl IndexMut<Coordinate> for Board {
    fn index_mut(&mut self, i: Coordinate) -> &mut BoardContent {
        &mut self.cells[i.index()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Move(Coordinate, Coordinate),
    Hurl(Coordinate, Coordinate),
    Shove(Coordinate, Coordinate, Vec<Coordinate>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    role: Role,
    name: String,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}

pub struct GameState {
    board: Board,
    players: [Player; 2],
    first_player_active: bool,
}

impl GameState {
    pub fn new(board: Board, player1_name: String, player2_name: String) -> Self {
        GameState {
            board: board,
            players: [Player { role: Role::Dwarf, name: player1_name, },
                      Player { role: Role::Troll, name: player2_name, },],
            first_player_active: true,
        }
    }
    
    pub fn new_default(player1_name: String, player2_name: String) -> Self {
        GameState::new(Board::default(), player1_name, player2_name)
    }

    pub fn active_player(&self) -> &Player {
        if self.first_player_active {
            &self.players[0]
        } else {
            &self.players[1]
        }
    }

    pub fn actions(&self, r: Role) {
        self.board.actions(r)
    }

    pub fn toggle_player(&mut self) {
        self.first_player_active = !self.first_player_active
    }
}

// pub struct ActionIterator<'a> {
//     cells: &'a Board,
//     role: Role,
// }

// impl Iterator for ActionIterator {
//     type Item = Action;

//     fn next(&mut self) -> Option<Action> {
//         if cell_index >= board.cells.size() {
            
//         }
//     }
// }

pub struct MoveIterator<'a> {
    board: &'a Board,
    start: Coordinate,
    ray: Ray,
}

impl<'a> MoveIterator<'a> {
    fn new(board: &'a Board, start: Coordinate, d: Direction) -> Self {
        let mut ray = Ray::new(start, d);
        ray.next();
        MoveIterator { board: board, start: start, ray: ray, }
    }
}

impl<'a> Iterator for MoveIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        self.ray.next().and_then(
            |end|
            if self.board[end].is_empty() {
                Some(Action::Move(self.start, end))
            } else {
                None
            })
    }
}

pub struct ShoveIterator<'a> {
    board: &'a Board,
    start: Coordinate,
    forward: Ray,
    backward: Ray,
}

impl<'a> ShoveIterator<'a> {
    fn new(board: &'a Board, start: Coordinate, d: Direction) -> Self {
        let mut forward = Ray::new(start, d);
        let backward = forward.reverse();
        forward.next();
        ShoveIterator { board: board, start: start, forward: forward, backward: backward, }
    }
}

impl<'a> Iterator for ShoveIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        loop {
            match (self.forward.next(), self.backward.next()) {
                (Some(end), Some(previous))
                    if self.board[end].is_empty() && self.board[previous].is_troll() => {
                        let mut captured = Vec::with_capacity(8);
                        for d in Direction::all() {
                            match end.to_direction(*d) {
                                Some(adjacent) if self.board[adjacent].is_dwarf() => {
                                    captured.push(adjacent)
                                },
                                _ => (),
                            }
                        }
                        if captured.is_empty() {
                            continue
                        } else {
                            return Some(Action::Shove(self.start, end, captured))
                        }
                    },
                _ => return None,
            }
        }
    }
}

pub struct HurlIterator<'a> {
    board: &'a Board,
    start: Coordinate,
    forward: Ray,
    backward: Ray,
    done: bool,
}

impl<'a> HurlIterator<'a> {
    pub fn new(board: &'a Board, start: Coordinate, d: Direction) -> Self {
        let mut forward = Ray::new(start, d);
        let backward = forward.reverse();
        forward.next();
        HurlIterator { board: board, start: start, forward: forward, backward: backward, done: false, }
    }
}

impl<'a> Iterator for HurlIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        if self.done {
            return None
        }
        self.done = true;
        loop {
            match (self.forward.next(), self.backward.next()) {
                (Some(end), Some(previous)) if self.board[previous].is_dwarf() => {
                    match self.board[end] {
                        BoardContent::Occupied(Token::Troll) => return Some(Action::Hurl(self.start, end)),
                        BoardContent::Empty => continue,
                        _ => return None,
                    }
                },
                _ => return None,
            }
        }
    }
}

// // For now, assume that we're on a standard Thud grid.
// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Coordinate(u8);

// // Number of cells in each row.
// const ROW_LENGTHS: [u8; 15] = [5, 7, 9, 11, 13, 15, 15, 15, 15, 15, 13, 11, 9, 7, 5];

// // The first cell of each row.
// const ROW_OFFSETS: [u8; 15] = [0, 5, 12, 21, 32, 45, 60, 75, 90, 105, 120, 133, 144, 153, 160];

// impl Coordinate {
//     pub fn new(c: u8) -> Self {
//         assert!(c < 165u8);
//         Coordinate(c)
//     }

//     pub fn offset(self) -> u8 {
//         let Coordinate(c) = self;
//         c
//     }

//     pub fn row(self) -> u8 {
//         let Coordinate(c) = self;
//         let mut i = 0u8;
//         loop {
//             if c < ROW_OFFSETS[i as usize] + ROW_LENGTHS[i as usize]{
//                 return i
//             }
//             i += 1;
//         }
//     }

//     pub fn column(self) -> u8 {
//         let Coordinate(c) = self;
//         let r = self.row();
//         c - ROW_OFFSETS[r as usize] + ROW_LENGTHS[r as usize]
//     }

//     pub fn left(self) -> Option<Self> {
//         let r = self.row();
//         let Coordinate(c) = self;
//         if c == ROW_OFFSETS[r as usize] {
//             None
//         } else {
//             Some(Coordinate::new(c - 1))
//         }
//     }

//     pub fn right(self) -> Option<Self> {
//         let r = self.row();
//         let Coordinate(c) = self;
//         if c == 165u8 || c == ROW_OFFSETS[(r + 1) as usize] {
//             None
//         } else {
//             Some(Coordinate::new(c + 1))
//         }
//     }

//     pub fn up(self) -> Option<Self> {
//         let row = self.row();
//         if row == 0u8 {
//             None
//         } else {
//             let col = self.column();
//             let prev_row_end = ROW_OFFSETS[(row - 1) as usize] + ROW_LENGTHS[(row - 1) as usize];
//             if prev_row_end < col {
//                 None
//             } else {
//                 Some(Coordinate::new(ROW_OFFSETS[(row - 1) as usize] + col))
//             }
//         }
//     }

//     pub fn up_left(self) -> Option<Self> {
//         self.up().and_then(|n| n.left())
//     }

//     pub fn up_right(self) -> Option<Self> {
//         self.up().and_then(|n| n.right())
//     }

//     pub fn down_left(self) -> Option<Self> {
//         self.down().and_then(|n| n.left())
//     }

//     pub fn down_right(self) -> Option<Self> {
//         self.down().and_then(|n| n.right())
//     }

//     pub fn down(self) -> Option<Self> {
//         let row = self.row();
//         if row == 14u8 {
//             None
//         } else {
//             let col = self.column();
//             if col > ROW_LENGTHS[(row + 1) as usize] {
//                 None
//             } else {
//                 match ROW_LENGTHS[row as usize].cmp(ROW_LENGTHS[(row + 1) as usize]) {
//                     Ordering::Less =>
//                         Some(Coordinate::new(ROW_OFFSETS[(row + 1) as usize] + col)),
//                     Ordering::Greater =>
//                         Some(Coordinate::new(ROW_OFFSETS[(row + 1) as usize] + col)),
//                     Ordering::Equal =>
//                         Some(Coordinate::new(ROW_OFFSETS[(row + 1) as usize] + col)),
                    
//                 }
//             }
//         }
//     }

//     /// Returns an iterator over the coordinates adjacent to `c`.
//     pub fn adjacent_coordinates_iter(self) -> AdjacentCoordinateIter {
//         AdjacentCoordinateIter {
//             index: 0u8,
//             coordinates: [self.up(), self.down(), self.left(), self.right()],
//         }
//     }
// }

// pub struct AdjacentCoordinateIter {
//     index: u8,
//     coordinates: [Option<Coordinate>; 4],
// }

// impl Iterator for AdjacentCoordinateIter {
//     type Item = Coordinate;

//     fn next(&mut self) -> Option<Coordinate> {
//         while self.index < 4u8 {
//             self.index += 1u8;
//             if self.coordinates[self.index as usize].is_some() {
//                 return self.coordinates[self.index as usize]
//             }
//         }
//         None
//     }
// }

// ///     The octagonal playing area consists of a 15 by 15 square board from
// ///     which a triangle of 15 squares in each corner has been removed. The
// ///     Thudstone is placed on the centre square of the board, where it remains
// ///     for the entire game and may not be moved onto or through. The eight
// ///     trolls are placed onto the eight squares orthogonally and diagonally
// ///     adjacent to the Thudstone and the thirty-two dwarfs are placed so as to
// ///     occupy all the perimeter spaces except for the four in the same
// ///     horizontal or vertical line as the Thudstone. One player takes control
// ///     of the dwarfs, the other controls the trolls. The dwarfs move first.
// ///
// /// This gives us 165 spaces, each of which may contain a piece.
// pub struct BoardState {
//     spaces: [Option<Piece>; 165],
// }

// impl BoardState {
//     /// Returns a freshly constructed board for a standard game of Thud.
//     pub fn new() -> Self {
//         let mut board = BoardState { spaces: [None; 165], };
//         // Place Dwarfs.
//         let mut c = Coordinate::new(0u8);
//         loop {
//             println!["{:?}", c];
//             if c.column() != 2 {
//                 board.set_piece_at(c, Some(Piece::Dwarf));
//             }
//             match c.right() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             match c.down_right() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             if c.row() != 7 {
//                 board.set_piece_at(c, Some(Piece::Dwarf));
//             }
//             match c.down() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             match c.down_left() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             if c.column() != 2 {
//                 board.set_piece_at(c, Some(Piece::Dwarf));
//             }
//             match c.left() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             match c.up_left() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         loop {
//             println!["{:?}", c];
//             if c.row() != 7 {
//                 board.set_piece_at(c, Some(Piece::Dwarf));
//             }
//             match c.up() {
//                 Some(new_c) => c = new_c,
//                 None => break,
//             }
//         }
//         // Place Thudstone.
//         c = Coordinate::new(82);
//         board.set_piece_at(c, Some(Piece::Thudstone));
//         // Place Trolls.
//         board.set_piece_at(c.up().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.down().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.left().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.right().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.up_left().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.up_right().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.down_left().unwrap(), Some(Piece::Troll));
//         board.set_piece_at(c.down_right().unwrap(), Some(Piece::Troll));
//         board
//     }

//     /// Returns the piece at `c`.
//     pub fn piece_at(&self, c: Coordinate) -> Option<Piece> {
//         let Coordinate(offset) = c;
//         self.spaces[offset as usize]
//     }

//     /// Sets the piece at `c`.
//     pub fn set_piece_at(&mut self, c: Coordinate, p: Option<Piece>) {
//         let Coordinate(offset) = c;
//         self.spaces[offset as usize] = p;
//     }
// }

// #[derive(Clone, Copy, Eq, PartialEq)]
// pub enum Player {
//     Dwarf,
//     Troll,
// }

// pub struct GameState {
//     current_player: Player,
//     actions: Vec<Action>,
//     board: BoardState,
// }

// macro_rules! build_dwarf_move_actions {
//     ($board: expr, $start: expr, $mutator_expr: expr, $result: ident) => (
//         {
//             let mutator = $mutator_expr;
//             let mut end = mutator($start);
//             loop {
//                 match end {
//                     Some(e) if $board.piece_at(e).is_none() => {
//                         $result.push(Action::Move($start, e));
//                         end = mutator(e);
//                     },
//                     _ => break,
//                 }
//             }
//         })
// }

// macro_rules! build_dwarf_hurl_actions {
//     ($board: expr, $start: expr, $forward_expr: expr, $backward_expr: expr, $result: ident) => (
//         {
//             let forward = $forward_expr;
//             let backward = $backward_expr;
//             let mut end = forward($start);
//             let mut line_start = $start;
//             loop {
//                 match end {
//                     Some(e) if $board.piece_at(e) == Some(Piece::Troll) => {  // Land on a troll.
//                         $result.push(Action::Hurl($start, e));
//                         break
//                     },
//                     Some(e) if $board.piece_at(e).is_none() => {  // No obstacles.
//                         match backward(line_start) {
//                             Some(s) if $board.piece_at(s) == Some(Piece::Dwarf) => {
//                                 // More dwarfs behind, so continue search.
//                                 line_start = s;
//                                 end = forward(e);
//                             },
//                             _ => break,  // No more dwarfs behind.
//                         }
//                     },
//                     _ => break,  // Ran off of end of board or hit a dwarf or the Thudstone.
//                 }
//             }
//         })
// }

// macro_rules! build_troll_move_actions {
//     ($board: expr, $start: expr, $mutator_expr: expr, $result: ident) => (
//         {
//             let mutator = $mutator_expr;
//             match mutator($start) {
//                 Some(e) if $board.piece_at(e).is_none() =>
//                     $result.push(Action::Move($start, e)),  // Nothing in the way.
//                 _ => (),  // Obstacle.
//             }
//         })
// }

// macro_rules! build_troll_shove_actions {
//     ($board: expr, $start: expr, $forward_expr: expr, $backward_expr: expr, $result: ident) => (
//         {
//             let forward = $forward_expr;
//             let backward = $backward_expr;
//             let mut end = forward($start);
//             let mut line_start = $start;
//             loop {
//                 match end {
//                     Some(e) if $board.piece_at(e) == None => {
//                         if e.up().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf)
//                             .or_else(|| e.down().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.left().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.right().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.up_left().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.up_right().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.down_left().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .or_else(|| e.down_right().and_then(|c| $board.piece_at(c)).map(|p| p == Piece::Dwarf))
//                             .unwrap_or(false) {
//                                 // At least one dwarf is adjacent to this space,
//                                 // so a shove that lands here will capture.
//                                 $result.push(Action::Shove($start, e));
//                                 match backward(line_start) {
//                                     Some(s) if $board.piece_at(s) == Some(Piece::Troll) => {
//                                         // More trolls behind, so continue search.
//                                         line_start = s;
//                                         end = forward(e);
//                                     },
//                                     _ => break,  // No more trolls behind.
//                                 }
//                             }
//                     },
//                     _ => break,  // Ran off of end of board or hit an occupied square.
//                 }
//             }
//         })
// }

// impl GameState {
//     pub fn new() -> Self {
//         GameState {
//             current_player: Player::Dwarf,
//             actions: vec![],
//             board: BoardState::new(),
//         }
//     }

//     pub fn player_actions(&self, p: Player) -> Vec<Action> {
//         let mut result = vec![];
//         match p {
//             Player::Dwarf =>
//                 for i in 0u8..165u8 {
//                     let start = Coordinate::new(i);
//                     if let Some(Piece::Dwarf) = self.board.piece_at(start) {
//                         // Move.
//                         build_dwarf_move_actions![
//                             self.board, start, |c: Coordinate| c.up(), result];
//                         build_dwarf_move_actions![
//                             self.board, start, |c: Coordinate| c.down(), result];
//                         build_dwarf_move_actions![
//                             self.board, start, |c: Coordinate| c.left(), result];
//                         build_dwarf_move_actions![
//                             self.board, start, |c: Coordinate| c.right(), result];
//                         build_dwarf_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_left(), result];
//                         build_dwarf_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_right(), result];
//                         build_dwarf_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_left(), result];
//                         build_dwarf_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_right(), result];
//                         // Hurl.
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up(), |c: Coordinate| c.down(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down(), |c: Coordinate| c.up(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.left(), |c: Coordinate| c.right(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.right(), |c: Coordinate| c.left(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_left(), |c: Coordinate| c.down_right(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_right(), |c: Coordinate| c.down_left(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_left(), |c: Coordinate| c.up_right(), result];
//                         build_dwarf_hurl_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_right(), |c: Coordinate| c.up_left(), result];
//                     }
//                 },
//             Player::Troll =>
//                 for i in 0u8..165u8 {
//                     let start = Coordinate::new(i);
//                     if let Some(Piece::Troll) = self.board.piece_at(start) {
//                         // Move.
//                         build_troll_move_actions![
//                             self.board, start, |c: Coordinate| c.up(), result];
//                         build_troll_move_actions![
//                             self.board, start, |c: Coordinate| c.down(), result];
//                         build_troll_move_actions![
//                             self.board, start, |c: Coordinate| c.left(), result];
//                         build_troll_move_actions![
//                             self.board, start, |c: Coordinate| c.right(), result];
//                         build_troll_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_left(), result];
//                         build_troll_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_right(), result];
//                         build_troll_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_left(), result];
//                         build_troll_move_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_right(), result];
//                         // Shove.
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up(), |c: Coordinate| c.down(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down(), |c: Coordinate| c.up(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.left(), |c: Coordinate| c.right(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.right(), |c: Coordinate| c.left(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_left(),
//                             |c: Coordinate| c.down_right(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.up_right(),
//                             |c: Coordinate| c.down_left(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_left(),
//                             |c: Coordinate| c.up_right(), result];
//                         build_troll_shove_actions![
//                             self.board, start,
//                             |c: Coordinate| c.down_right(),
//                             |c: Coordinate| c.up_left(), result];
//                     }
//                 },
//             }
//         result
//     }

//     pub fn player(&self) -> Player {
//         self.current_player
//     }

//     pub fn advance_player(&mut self) {
//         self.current_player = match self.current_player {
//             Player::Dwarf => Player::Troll,
//             Player::Troll => Player::Dwarf,
//         }
//     }

//     fn remove_adjacent_dwarves(&mut self, end: Coordinate) {
//         for adjacency in [end.up(), end.down(), end.left(), end.right(),
//                           end.up_left(), end.up_right(),
//                           end.down_left(), end.down_right()].iter() {
//             if let Some((e, Some(Piece::Dwarf))) = adjacency.map(|c| (c, self.board.piece_at(c))) {
//                 self.board.set_piece_at(e, None);
//             }
//         }
//     }

//     pub fn take_action(&mut self, a: Action) {
//         // TODO: validation.
//         self.actions.push(a);
//         match a {
//             Action::Move(start, end) => {
//                 let p = self.board.piece_at(start);
//                 self.board.set_piece_at(end, p);
//                 self.board.set_piece_at(start, None);
//                 if self.current_player == Player::Troll {
//                     self.remove_adjacent_dwarves(end);
//                 }
//             },
//             Action::Hurl(start, end) => {
//                 let p = self.board.piece_at(start);
//                 self.board.set_piece_at(end, p);
//                 self.board.set_piece_at(start, None);
//             },
//             Action::Shove(start, end) => {
//                 let p = self.board.piece_at(start);
//                 self.board.set_piece_at(end, p);
//                 self.board.set_piece_at(start, None);
//                 self.remove_adjacent_dwarves(end);
//             },
//         }
//     }
// }

// #[derive(Clone, Copy, Eq, PartialEq)]
// pub enum Action {
//     Move(Coordinate, Coordinate),
//     Hurl(Coordinate, Coordinate),
//     Shove(Coordinate, Coordinate),
// }
