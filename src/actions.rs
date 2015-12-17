use ::board;

use std::iter::{Chain, FlatMap, Iterator, Take};
use std::slice;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Move(board::Coordinate, board::Coordinate),
    Hurl(board::Coordinate, board::Coordinate),
    Shove(board::Coordinate, board::Coordinate, Vec<board::Coordinate>),
    Concede,
}

pub struct DwarfDirectionConsumer<'a> {
    board: &'a board::Cells,
    position: board::Coordinate,
}

impl<'a> DwarfDirectionConsumer<'a> {
    pub fn new(board: &'a board::Cells, position: board::Coordinate) -> Self {
        DwarfDirectionConsumer { board: board, position: position, }
    }
}

impl<'a> FnOnce<(&'a board::Direction,)> for DwarfDirectionConsumer<'a> {
    type Output = Chain<MoveIterator<'a>, HurlIterator<'a>>;

    extern "rust-call" fn call_once(self, (d,): (&'a board::Direction,)) -> Chain<MoveIterator<'a>, HurlIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d)
            .chain(HurlIterator::new(self.board, self.position, *d))
    }
}

impl<'a> FnMut<(&'a board::Direction,)> for DwarfDirectionConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (d,): (&'a board::Direction,)) -> Chain<MoveIterator<'a>, HurlIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d)
            .chain(HurlIterator::new(self.board, self.position, *d))
    }
}

pub struct DwarfCoordinateConsumer<'a> {
    board: &'a board::Cells,
}

impl<'a> DwarfCoordinateConsumer<'a> {
    pub fn new(board: &'a board::Cells) -> Self {
        DwarfCoordinateConsumer { board: board, }
    }
}

impl<'a> FnOnce<(board::Coordinate,)> for DwarfCoordinateConsumer<'a> {
    type Output = FlatMap<slice::Iter<'a, board::Direction>,
                          Chain<MoveIterator<'a>, HurlIterator<'a>>,
                          DwarfDirectionConsumer<'a>>;

    extern "rust-call" fn call_once(self, (c,): (board::Coordinate,)) -> FlatMap<slice::Iter<'a, board::Direction>,
                                                                              Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                                                              DwarfDirectionConsumer<'a>> {
        board::Direction::all()
            .into_iter()
            .flat_map(DwarfDirectionConsumer { board: self.board, position: c, })
    }
}

impl<'a> FnMut<(board::Coordinate,)> for DwarfCoordinateConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (c,): (board::Coordinate,)) -> FlatMap<slice::Iter<'a, board::Direction>,
                                                                                  Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                                                                  DwarfDirectionConsumer<'a>> {
        board::Direction::all()
            .into_iter()
            .flat_map(DwarfDirectionConsumer { board: self.board, position: c, })
    }
}

pub type DwarfActionIter<'a> = FlatMap<board::OccupiedCellsIter<'a>,
                                       FlatMap<slice::Iter<'a, board::Direction>,
                                               Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                               DwarfDirectionConsumer<'a>>,
                                       DwarfCoordinateConsumer<'a>>;

pub struct TrollDirectionConsumer<'a> {
    board: &'a board::Cells,
    position: board::Coordinate,
}

impl<'a> TrollDirectionConsumer<'a> {
    pub fn new(board: &'a board::Cells, position: board::Coordinate) -> Self {
        TrollDirectionConsumer { board: board, position: position, }
    }
}

impl<'a> FnOnce<(&'a board::Direction,)> for TrollDirectionConsumer<'a> {
    type Output = Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>;

    extern "rust-call" fn call_once(self, (d,): (&'a board::Direction,)) -> Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d).take(1)
            .chain(ShoveIterator::new(self.board, self.position, *d))
    }
}

impl<'a> FnMut<(&'a board::Direction,)> for TrollDirectionConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (d,): (&'a board::Direction,)) -> Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>> {
        MoveIterator::new(self.board, self.position, *d).take(1)
            .chain(ShoveIterator::new(self.board, self.position, *d))
    }
}

pub struct TrollCoordinateConsumer<'a> {
    board: &'a board::Cells,
}

impl<'a> TrollCoordinateConsumer<'a> {
    pub fn new(board: &'a board::Cells) -> Self {
        TrollCoordinateConsumer { board: board, }
    }
}

impl<'a> FnOnce<(board::Coordinate,)> for TrollCoordinateConsumer<'a> {
    type Output = FlatMap<slice::Iter<'a, board::Direction>,
                          Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                          TrollDirectionConsumer<'a>>;

    extern "rust-call" fn call_once(self, (c,): (board::Coordinate,)) -> FlatMap<slice::Iter<'a, board::Direction>,
                                                                          Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                                                          TrollDirectionConsumer<'a>> {
        board::Direction::all()
            .into_iter()
            .flat_map(TrollDirectionConsumer { board: self.board, position: c, })
    }
}

impl<'a> FnMut<(board::Coordinate,)> for TrollCoordinateConsumer<'a> {
    extern "rust-call" fn call_mut(&mut self, (c,): (board::Coordinate,)) -> FlatMap<slice::Iter<'a, board::Direction>,
                                                                              Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                                                              TrollDirectionConsumer<'a>> {
        board::Direction::all()
            .into_iter()
            .flat_map(TrollDirectionConsumer { board: self.board, position: c, })
    }
}

pub type TrollActionIter<'a> = FlatMap<board::OccupiedCellsIter<'a>,
                                       FlatMap<slice::Iter<'a, board::Direction>,
                                               Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                               TrollDirectionConsumer<'a>>,
                                       TrollCoordinateConsumer<'a>>;

pub type DwarfPositionActionIter<'a> = FlatMap<slice::Iter<'a, board::Direction>,
                                               Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                               DwarfDirectionConsumer<'a>>;

pub type TrollPositionActionIter<'a> = FlatMap<slice::Iter<'a, board::Direction>,
                                               Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                               TrollDirectionConsumer<'a>>;

enum ActionIteratorInner<'a> {
    Empty,
    Dwarf(DwarfActionIter<'a>),  // All dwarf actions on the board.
    Troll(TrollActionIter<'a>),  // All troll actions on the board.
    DwarfPosition(DwarfPositionActionIter<'a>),  // All actions for a position.
    TrollPosition(TrollPositionActionIter<'a>),  // All actions for a position.
}

/// Iterates over player actions on a board.
pub struct ActionIterator<'a> {
    inner: ActionIteratorInner<'a>,
}

impl<'a> ActionIterator<'a> {
    pub fn empty() -> Self {
        ActionIterator { inner: ActionIteratorInner::Empty, }
    }

    pub fn for_dwarf(wrapped: DwarfActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::Dwarf(wrapped), }
    }

    pub fn for_troll(wrapped: TrollActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::Troll(wrapped), }
    }

    pub fn for_dwarf_position(wrapped: DwarfPositionActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::DwarfPosition(wrapped), }
    }

    pub fn for_troll_position(wrapped: TrollPositionActionIter<'a>) -> Self {
        ActionIterator { inner: ActionIteratorInner::TrollPosition(wrapped), }
    }
}

impl<'a> Iterator for ActionIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        match self.inner {
            ActionIteratorInner::Empty => None,
            ActionIteratorInner::Dwarf(ref mut x) => x.next(),
            ActionIteratorInner::Troll(ref mut x) => x.next(),
            ActionIteratorInner::DwarfPosition(ref mut x) => x.next(),
            ActionIteratorInner::TrollPosition(ref mut x) => x.next(),
        }
    }
}

/// Iterates over move actions that may be made on a board.
///
///     Any dwarf is moved like a chess queen, any number of squares in any
///     orthogonal or diagonal direction, but not onto or through any other
///     piece, whether Thudstone, dwarf, or troll.
///
///     Any troll is moved like a chess king, one square in any orthogonal or
///     diagonal direction onto an empty square. After the troll has been moved,
///     only a single dwarf on the eight squares adjacent to the moved troll may
///     optionally be immediately captured and removed from the board, at the
///     troll player's discretion.
///
/// To limit the number of squares moved in the case of moving a troll, limit a
/// `Moveiterator` with its `take()` method.
pub struct MoveIterator<'a> {
    board: &'a board::Cells,
    start: board::Coordinate,
    ray: board::Ray,
}

impl<'a> MoveIterator<'a> {
    /// Creates a new iterator that will iterate over all move actions for the
    /// piece at `start` in the direction `d`, for arbitrarily many spaces
    /// (until the edge of the board's playable space is reached).
    fn new(board: &'a board::Cells, start: board::Coordinate,
           d: board::Direction) -> Self {
        let mut ray = board::Ray::new(start, d);
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

/// Iterates over shove actions (Troll capturing moves) that may be made on a
/// board.
///
///     Anywhere there is a straight (orthogonal or diagonal) line of adjacent
///     trolls on the board, they may shove the endmost troll in the direction
///     continuing the line, up to as many spaces as there are trolls in the
///     line. As in a normal move, the troll may not land on an occupied square,
///     and any (all) dwarfs in the eight squares adjacent to its final position
///     may immediately be captured. Trolls may only make a shove if by doing so
///     they capture at least one dwarf.
pub struct ShoveIterator<'a> {
    board: &'a board::Cells,
    start: board::Coordinate,
    forward: board::Ray,
    backward: board::Ray,
}

impl<'a> ShoveIterator<'a> {
    /// Creates an iterator that will iterate over all shove actions for the
    /// piece at `start` in the direction `d`.
    fn new(board: &'a board::Cells, start: board::Coordinate, d: board::Direction) -> Self {
        let mut forward = board::Ray::new(start, d);
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
                        for d in board::Direction::all() {
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

/// Iterates over hurl actions (Dwarf capturing actions) that may be performed
/// on a `Board`.
///
///     Anywhere there is a straight (orthogonal or diagonal) line of adjacent
///     dwarfs on the board, they may hurl the front dwarf in the direction
///     continuing the line, as long as the space between the lead dwarf and the
///     troll is less than the number of dwarfs in the line. This is different
///     from a normal move in that the dwarf is permitted to land on a square
///     containing a troll, in which case the troll is removed from the board
///     and the dwarf takes his place. This may only be done if the endmost
///     dwarf can land on a troll by moving in the direction of the line at most
///     as many spaces as there are dwarfs in the line. Since a single dwarf is
///     a line of one in any direction, a dwarf may always move one space to
///     capture a troll on an immediately adjacent square.
pub struct HurlIterator<'a> {
    board: &'a board::Cells,
    start: board::Coordinate,
    forward: board::Ray,
    backward: board::Ray,
    done: bool,
}

impl<'a> HurlIterator<'a> {
    /// Creates an iterator that will iterate over all hurl actions for the
    /// piece at `start` in the direction `d`.
    pub fn new(board: &'a board::Cells, start: board::Coordinate, d: board::Direction) -> Self {
        let mut forward = board::Ray::new(start, d);
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
                        board::Content::Occupied(board::Token::Troll) => return Some(Action::Hurl(self.start, end)),
                        board::Content::Empty => continue,
                        _ => return None,
                    }
                },
                _ => return None,
            }
        }
    }
}