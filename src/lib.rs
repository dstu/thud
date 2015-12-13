#![feature(associated_type_defaults)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

pub mod console;

use std::iter::{Chain, FlatMap, Iterator, Take};
use std::ops::{Index, IndexMut};

use std::fmt;
use std::slice;

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

    pub fn actions<'s>(&'s self, r: Role) -> ActionIterator<'s> {
        let occupied_cells = self.occupied_iter(r);
        match r {
            Role::Dwarf =>
                ActionIterator::for_dwarf(
                    occupied_cells.flat_map(DwarfCoordinateConsumer { board: self, })),
                //  occupied_cells.flat_map(|position| {
                //         Direction::all()
                //             .into_iter()
                //             .flat_map(|d| (MoveIterator::new(self, position, *d)
                //                            .chain(HurlIterator::new(self, position, *d))))
                // })),
            Role::Troll =>
                ActionIterator::for_troll(
                    occupied_cells.flat_map(TrollCoordinateConsumer { board: self, })),
                    // occupied_cells.flat_map(|position| {
                    //     Direction::all()
                    //         .into_iter()
                    //         .flat_map(|d| (MoveIterator::new(self, position, *d).take(1)
                    //                        .chain(ShoveIterator::new(self, position, *d))))
                    // })),
        }
    }

    pub fn do_action(&mut self, a: &Action) {
        match a {
            &Action::Move(start, end) => {
                self[end] = self[start];
                self[start] = BoardContent::Empty;
            },
            &Action::Hurl(start, end) => {
                self[end] = self[start];
                self[start] = BoardContent::Empty;
            },
            &Action::Shove(start, end, ref captured) => {
                self[end] = self[start];
                self[start] = BoardContent::Empty;
                for c in captured {
                    self[*c] = BoardContent::Empty;
                }
            },
        }
    }

    pub fn occupied_iter<'s>(&'s self, r: Role) -> OccupiedCellsIter<'s> {
        OccupiedCellsIter { board: self, role: r, index: 0, }
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

pub struct OccupiedCellsIter<'a> {
    board: &'a Board,
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
                    BoardContent::Occupied(t) if t.role() == Some(self.role) =>
                        return Some(coordinate),
                    _ => continue,
                }
            }
        }
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

    pub fn actions<'s>(&'s self, r: Role) -> ActionIterator<'s> {
        self.board.actions(r)
    }

    pub fn toggle_player(&mut self) {
        self.first_player_active = !self.first_player_active
    }

    pub fn do_action(&mut self, a: &Action) {
        self.board.do_action(a);
        self.toggle_player();
    }
}

struct DwarfDirectionConsumer<'a> {
    board: &'a Board,
    position: Coordinate,
}

impl<'a> FnOnce<(&'a Direction,)> for DwarfDirectionConsumer<'a> {
    type Output = Chain<MoveIterator<'a>, HurlIterator<'a>>;

    extern "rust-call" fn call_once(self, (d,): (&'a Direction,)) -> Chain<MoveIterator<'a>, HurlIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d)
            .chain(HurlIterator::new(self.board, self.position, *d))
    }
}

impl<'a> FnMut<(&'a Direction,)> for DwarfDirectionConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (d,): (&'a Direction,)) -> Chain<MoveIterator<'a>, HurlIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d)
            .chain(HurlIterator::new(self.board, self.position, *d))
    }
}

struct DwarfCoordinateConsumer<'a> {
    board: &'a Board,
}

impl<'a> FnOnce<(Coordinate,)> for DwarfCoordinateConsumer<'a> {
    type Output = FlatMap<slice::Iter<'a, Direction>,
                          Chain<MoveIterator<'a>, HurlIterator<'a>>,
                          DwarfDirectionConsumer<'a>>;

    extern "rust-call" fn call_once(self, (c,): (Coordinate,)) -> FlatMap<slice::Iter<'a, Direction>,
                                                                              Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                                                              DwarfDirectionConsumer<'a>> {
        Direction::all()
            .into_iter()
            .flat_map(DwarfDirectionConsumer { board: self.board, position: c, })
    }
}

impl<'a> FnMut<(Coordinate,)> for DwarfCoordinateConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (c,): (Coordinate,)) -> FlatMap<slice::Iter<'a, Direction>,
                                                                                  Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                                                                  DwarfDirectionConsumer<'a>> {
        Direction::all()
            .into_iter()
            .flat_map(DwarfDirectionConsumer { board: self.board, position: c, })
    }
}

type DwarfActionIter<'a> = FlatMap<OccupiedCellsIter<'a>,
                               FlatMap<slice::Iter<'a, Direction>,
                                       Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                       DwarfDirectionConsumer<'a>>,
                               DwarfCoordinateConsumer<'a>>;

struct TrollDirectionConsumer<'a> {
    board: &'a Board,
    position: Coordinate,
}

impl<'a> FnOnce<(&'a Direction,)> for TrollDirectionConsumer<'a> {
    type Output = Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>;

    extern "rust-call" fn call_once(self, (d,): (&'a Direction,)) -> Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d).take(1)
            .chain(ShoveIterator::new(self.board, self.position, *d))
    }
}

impl<'a> FnMut<(&'a Direction,)> for TrollDirectionConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (d,): (&'a Direction,)) -> Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d).take(1)
            .chain(ShoveIterator::new(self.board, self.position, *d))
    }
}

struct TrollCoordinateConsumer<'a> {
    board: &'a Board,
}

impl<'a> FnOnce<(Coordinate,)> for TrollCoordinateConsumer<'a> {
    type Output = FlatMap<slice::Iter<'a, Direction>,
                          Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                          TrollDirectionConsumer<'a>>;

    extern "rust-call" fn call_once(self, (c,): (Coordinate,)) -> FlatMap<slice::Iter<'a, Direction>,
                                                                          Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                                                          TrollDirectionConsumer<'a>> {
        Direction::all()
            .into_iter()
            .flat_map(TrollDirectionConsumer { board: self.board, position: c, })
    }
}

impl<'a> FnMut<(Coordinate,)> for TrollCoordinateConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (c,): (Coordinate,)) -> FlatMap<slice::Iter<'a, Direction>,
                                                                              Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                                                              TrollDirectionConsumer<'a>> {
        Direction::all()
            .into_iter()
            .flat_map(TrollDirectionConsumer { board: self.board, position: c, })
    }
}

type TrollActionIter<'a> = FlatMap<OccupiedCellsIter<'a>,
                               FlatMap<slice::Iter<'a, Direction>,
                                       Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                       TrollDirectionConsumer<'a>>,
                               TrollCoordinateConsumer<'a>>;

enum ActionIteratorInner<'a> {
    Dwarf(DwarfActionIter<'a>),
    Troll(TrollActionIter<'a>),
}

pub struct ActionIterator<'a> {
    inner: ActionIteratorInner<'a>,
}

impl<'a> ActionIterator<'a> {
    fn for_dwarf(wrapped: DwarfActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::Dwarf(wrapped), }
    }

    fn for_troll(wrapped: TrollActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::Troll(wrapped), }
    }
}

impl<'a> Iterator for ActionIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        match self.inner {
            ActionIteratorInner::Dwarf(ref mut x) => x.next(),
            ActionIteratorInner::Troll(ref mut x) => x.next(),
        }
    }
}

// pub trait ActionIterator<'a>: Iterator<Item=Action> {
//     fn next(&mut self) -> Option<Action>;

//     fn chain<J: ActionIterator<'a>>(self, other: J) -> ActionChain<'a, Self, J> {
//         ActionChain { front: self, back: other, state: ChainState::Both, }
//     }

//     fn take(self, count: usize) -> ActionTake<'a, Self> {
//         ActionTake { wrapped: self, count: count, }
//     }
// }

// impl<'a> Iterator for ActionIterator<'a> {
//     type Item = Action;

//     fn next(&mut self) -> Option<Action> {
//         self.next_action()
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

// enum ChainState {
//     Both,
//     Back,
// }

// pub struct ActionChain<'a, I: ActionIterator<'a>, J: ActionIterator<'a>> {
//     a: PhantomData<&'a ()>,
//     front: I,
//     back: J,
//     state: ChainState,
// }

// impl<'a, I: ActionIterator<'a>, J: ActionIterator<'a>> ActionIterator<'a> for ActionChain<'a, I, J> {
//     fn next(&mut self) -> Option<Action> {
//         match self.state {
//             ChainState::Both => match self.front.next() {
//                 s @ Some(..) => s,
//                 None => {
//                     self.state = ChainState::Back;
//                     self.back.next()
//                 },
//             },
//             ChainState::Back => self.back.next(),
//         }
//     }
// }

// pub struct ActionTake<'a, I: ActionIterator<'a>> {
//     a: PhantomData<&'a ()>,
//     wrapped: I,
//     count: usize,
// }

// impl<'a, I: ActionIterator<'a>> ActionIterator<'a> for ActionTake<'a, I> {
//     fn next(&mut self) -> Option<Action> {
//         if self.count != 0 {
//             self.count -= 1;
//             self.wrapped.next()
//         } else {
//             None
//         }
//     }
// }
