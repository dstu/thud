use super::board;
use super::coordinate::{Coordinate, Direction};
use super::end;

use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::iter::{Chain, FlatMap, Iterator, Take};
use std::slice;

#[derive(Clone, Copy, Hash)]
pub enum Action {
    Move(Coordinate, Coordinate),
    Hurl(Coordinate, Coordinate),
    Shove(Coordinate, Coordinate, u8, [Coordinate; 7]),
    ProposeEnd,
    HandleEndProposal(end::Decision),
}

impl Action {
    pub fn is_move(&self) -> bool {
        match self {
            &Action::Move(_, _) => true,
            _ => false,
        }
    }

    pub fn is_hurl(&self) -> bool {
        match self {
            &Action::Hurl(_, _) => true,
            _ => false,
        }
    }

    pub fn is_shove(&self) -> bool {
        match self {
            &Action::Shove(_, _, _, _) => true,
            _ => false,
        }
    }

    pub fn source(&self) -> Option<Coordinate> {
        match self {
            &Action::Move(s, _) => Some(s),
            &Action::Hurl(s, _) => Some(s),
            &Action::Shove(s, _, _, _) => Some(s),
            _ => None,
        }
    }

    pub fn target(&self) -> Option<Coordinate> {
        match self {
            &Action::Move(_, t) => Some(t),
            &Action::Hurl(_, t) => Some(t),
            &Action::Shove(_, t, _, _) => Some(t),
            _ => None,
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Action::Move(start, end) => write!(f, "Move({:?}, {:?})", start, end),
            &Action::Hurl(start, end) => write!(f, "Hurl({:?}, {:?})", start, end),
            &Action::Shove(start, end, capture_count, captured) => {
                try!(write!(f, "Shove({:?}, {:?}, [", start, end));
                try!(write!(f, "{:?}", captured[0]));
                for i in 1..capture_count {
                    try!(write!(f, ", {:?}", captured[i as usize]));
                }
                write!(f, "])")
            },
            &Action::ProposeEnd => write!(f, "ProposeEnd"),
            &Action::HandleEndProposal(d) => write!(f, "HandleEndProposal({:?})", d),
        }
    }
}

impl PartialEq<Action> for Action {
    fn eq(&self, rhs: &Action) -> bool {
        match (*self, *rhs) {
            (Action::Move(a_start, a_end), Action::Move(b_start, b_end)) =>
                a_start == b_start && a_end == b_end,
            (Action::Hurl(a_start, a_end), Action::Hurl(b_start, b_end)) =>
                a_start == b_start && a_end == b_end,
            (Action::Shove(a_start, a_end, a_capture_count, a_captures),
             Action::Shove(b_start, b_end, b_capture_count, b_captures))
                if a_start == b_start && a_end == b_end && a_capture_count == b_capture_count => {
                    let mut a_captures_sorted = a_captures.clone();
                    a_captures_sorted.sort();
                    let mut b_captures_sorted = b_captures.clone();
                    b_captures_sorted.sort();
                    a_captures_sorted == b_captures_sorted
                },
            (Action::ProposeEnd, Action::ProposeEnd) => true,
            (Action::HandleEndProposal(d1), Action::HandleEndProposal(d2)) => d1 == d2,
            _ => false,
        }
    }
}

impl Eq for Action { }

#[macro_export] macro_rules! move_literal {
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr)) =>
        ($crate::Action::Move(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col)));
}

#[macro_export] macro_rules! shove_literal {
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            1,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            2,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            3,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            4,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            5,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr),
      ($capture_6_row: expr, $capture_6_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            6,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_6_row, $capture_6_col),
             $crate::coordinate::Coordinate::new_unchecked(7, 7),]));
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr),
      ($capture_6_row: expr, $capture_6_col: expr),
      ($capture_7_row: expr, $capture_7_col: expr)]) =>
        ($crate::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            7,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_6_row, $capture_6_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_7_row, $capture_7_col),,]));
}

pub struct DwarfDirectionConsumer<'a> {
    board: &'a board::Cells,
    position: Coordinate,
}

impl<'a> DwarfDirectionConsumer<'a> {
    pub fn new(board: &'a board::Cells, position: Coordinate) -> Self {
        DwarfDirectionConsumer { board: board, position: position, }
    }
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

pub struct DwarfCoordinateConsumer<'a> {
    board: &'a board::Cells,
}

impl<'a> DwarfCoordinateConsumer<'a> {
    pub fn new(board: &'a board::Cells) -> Self {
        DwarfCoordinateConsumer { board: board, }
    }
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

pub type DwarfActionIter<'a> = FlatMap<board::OccupiedCellsIter<'a>,
                                       FlatMap<slice::Iter<'a, Direction>,
                                               Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                               DwarfDirectionConsumer<'a>>,
                                       DwarfCoordinateConsumer<'a>>;

pub struct TrollDirectionConsumer<'a> {
    board: &'a board::Cells,
    position: Coordinate,
}

impl<'a> TrollDirectionConsumer<'a> {
    pub fn new(board: &'a board::Cells, position: Coordinate) -> Self {
        TrollDirectionConsumer { board: board, position: position, }
    }
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

pub struct TrollCoordinateConsumer<'a> {
    board: &'a board::Cells,
}

impl<'a> TrollCoordinateConsumer<'a> {
    pub fn new(board: &'a board::Cells) -> Self {
        TrollCoordinateConsumer { board: board, }
    }
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

pub type TrollActionIter<'a> = FlatMap<board::OccupiedCellsIter<'a>,
                                       FlatMap<slice::Iter<'a, Direction>,
                                               Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                               TrollDirectionConsumer<'a>>,
                                       TrollCoordinateConsumer<'a>>;

pub type DwarfPositionActionIter<'a> = FlatMap<slice::Iter<'a, Direction>,
                                               Chain<MoveIterator<'a>, HurlIterator<'a>>,
                                               DwarfDirectionConsumer<'a>>;

pub type TrollPositionActionIter<'a> = FlatMap<slice::Iter<'a, Direction>,
                                               Chain<Take<MoveIterator<'a>>, ShoveIterator<'a>>,
                                               TrollDirectionConsumer<'a>>;

enum ActionIteratorInner<'a> {
    Empty,
    HandleEndProposal(slice::Iter<'static, end::Decision>),  // Opponent has offered to terminate game.
    Dwarf(DwarfActionIter<'a>),  // All dwarf actions on the board.
    Troll(TrollActionIter<'a>),  // All troll actions on the board.
    DwarfPosition(DwarfPositionActionIter<'a>),  // All actions for a board position.
    TrollPosition(TrollPositionActionIter<'a>),  // All actions for a board position.
}

/// Iterates over player actions on a board.
pub struct ActionIterator<'a> {
    end_proposal_allowed: bool,
    end_proposal_generated: bool,
    inner: ActionIteratorInner<'a>,
}

impl<'a> ActionIterator<'a> {
    pub fn empty() -> Self {
        ActionIterator {
            end_proposal_allowed: false,
            end_proposal_generated: false,
            inner: ActionIteratorInner::Empty,
        }
    }

    pub fn accept_or_decline_end() -> Self {
        ActionIterator {
            end_proposal_allowed: false,
            end_proposal_generated: false,
            inner: ActionIteratorInner::HandleEndProposal(end::Decision::all().iter()),
        }
    }

    pub fn for_dwarf(allow_end_proposal: bool, wrapped: DwarfActionIter<'a>) -> Self {
        ActionIterator {
            end_proposal_allowed: allow_end_proposal,
            end_proposal_generated: false,
            inner: ActionIteratorInner::Dwarf(wrapped),
        }
    }

    pub fn for_troll(allow_end_proposal: bool, wrapped: TrollActionIter<'a>) -> Self {
        ActionIterator {
            end_proposal_allowed: allow_end_proposal,
            end_proposal_generated: false,
            inner: ActionIteratorInner::Troll(wrapped),
        }
    }

    pub fn for_dwarf_position(wrapped: DwarfPositionActionIter<'a>) -> Self {
        ActionIterator {
            end_proposal_allowed: false,
            end_proposal_generated: false,
            inner: ActionIteratorInner::DwarfPosition(wrapped),
        }
    }

    pub fn for_troll_position(wrapped: TrollPositionActionIter<'a>) -> Self {
        ActionIterator {
            end_proposal_allowed: false,
            end_proposal_generated: false,
            inner: ActionIteratorInner::TrollPosition(wrapped),
        }
    }
}

impl<'a> Iterator for ActionIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        if self.end_proposal_allowed {
            if !self.end_proposal_generated {
                self.end_proposal_generated = true;
                return Some(Action::ProposeEnd)
            }
        }
        match self.inner {
            ActionIteratorInner::Empty => None,
            ActionIteratorInner::HandleEndProposal(ref mut x) =>
                x.next().map(|&x| Action::HandleEndProposal(x)),
            ActionIteratorInner::Dwarf(ref mut x) => x.next(),
            ActionIteratorInner::Troll(ref mut x) => x.next(),
            ActionIteratorInner::DwarfPosition(ref mut x) => x.next(),
            ActionIteratorInner::TrollPosition(ref mut x) => x.next(),
        }
    }
}

/// Iterates over move actions that may be made on a board.
///
/// Any dwarf is moved like a chess queen, any number of squares in any
/// orthogonal or diagonal direction, but not onto or through any other piece,
/// whether Thudstone, dwarf, or troll.
///
/// Any troll is moved like a chess king, one square in any orthogonal or
/// diagonal direction onto an empty square. After the troll has been moved,
/// only a single dwarf on the eight squares adjacent to the moved troll may
/// optionally be immediately captured and removed from the board, at the troll
/// player's discretion.
///
/// To limit the number of squares moved in the case of moving a troll, limit a
/// `Moveiterator` with its `take()` method.
pub struct MoveIterator<'a> {
    board: &'a board::Cells,
    start: Coordinate,
    ray: board::Ray,
}

impl<'a> MoveIterator<'a> {
    /// Creates a new iterator that will iterate over all move actions for the
    /// piece at `start` in the direction `d`, for arbitrarily many spaces
    /// (until the edge of the board's playable space is reached).
    fn new(board: &'a board::Cells, start: Coordinate,
           d: Direction) -> Self {
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
/// Anywhere there is a straight (orthogonal or diagonal) line of adjacent
/// trolls on the board, they may shove the endmost troll in the direction
/// continuing the line, up to as many spaces as there are trolls in the
/// line. As in a normal move, the troll may not land on an occupied square, and
/// any (all) dwarfs in the eight squares adjacent to its final position may
/// immediately be captured. Trolls may only make a shove if by doing so they
/// capture at least one dwarf.
pub struct ShoveIterator<'a> {
    board: &'a board::Cells,
    start: Coordinate,
    forward: board::Ray,
    backward: board::Ray,
}

impl<'a> ShoveIterator<'a> {
    /// Creates an iterator that will iterate over all shove actions for the
    /// piece at `start` in the direction `d`.
    fn new(board: &'a board::Cells, start: Coordinate, d: Direction) -> Self {
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
                        let mut captured = [coordinate_literal!(7, 7); 7];
                        let mut i = 0u8;
                        for d in Direction::all() {
                            match end.to_direction(*d) {
                                Some(adjacent) if self.board[adjacent].is_dwarf() => {
                                    // trace!("ShoveIterator: found shove from {:?} to {:?} that captures {:?} at {:?}",
                                    //        self.start, end, self.board[adjacent], adjacent);
                                    captured[i as usize] = adjacent;
                                    i += 1;
                                },
                                _ => (),
                            }
                        }
                        if i == 0 {
                            continue
                        } else {
                            return Some(Action::Shove(self.start, end, i, captured))
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
/// Anywhere there is a straight (orthogonal or diagonal) line of adjacent
/// dwarfs on the board, they may hurl the front dwarf in the direction
/// continuing the line, as long as the space between the lead dwarf and the
/// troll is less than the number of dwarfs in the line. This is different from
/// a normal move in that the dwarf is permitted to land on a square containing
/// a troll, in which case the troll is removed from the board and the dwarf
/// takes his place. This may only be done if the endmost dwarf can land on a
/// troll by moving in the direction of the line at most as many spaces as there
/// are dwarfs in the line. Since a single dwarf is a line of one in any
/// direction, a dwarf may always move one space to capture a troll on an
/// immediately adjacent square.
pub struct HurlIterator<'a> {
    board: &'a board::Cells,
    start: Coordinate,
    forward: board::Ray,
    backward: board::Ray,
    done: bool,
}

impl<'a> HurlIterator<'a> {
    /// Creates an iterator that will iterate over all hurl actions for the
    /// piece at `start` in the direction `d`.
    pub fn new(board: &'a board::Cells, start: Coordinate, d: Direction) -> Self {
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

#[cfg(test)]
mod test {
    use ::actions::Action;
    use ::board;
    use ::end;
    use ::state::State;
    use ::Role;

    #[test]
    fn troll_can_move() {
        let state = State::<board::TranspositionalEquivalence>::new(
            board::decode_board(r#"
....._____.....
...._______....
..._________...
..___________..
._____________.
_____________dT
_____________dd
_______O_______
_______________
_______________
._____________.
..___________..
..._________...
...._______....
....._____.....
"#));
        let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
        assert!(!actions.is_empty());
        assert_eq!(Action::ProposeEnd, Action::ProposeEnd);
        assert_eq!(actions,
                   vec!(Action::ProposeEnd,
                        Action::Move(coordinate_literal!(5, 14),
                                     coordinate_literal!(4, 13)),
                        Action::Shove(coordinate_literal!(5, 14),
                                      coordinate_literal!(4, 13),
                                      1,
                                      [coordinate_literal!(5, 13),
                                       coordinate_literal!(7, 7),
                                       coordinate_literal!(7, 7),
                                       coordinate_literal!(7, 7),
                                       coordinate_literal!(7, 7),
                                       coordinate_literal!(7, 7),
                                       coordinate_literal!(7, 7)])));
    }

    #[test]
    fn troll_cant_move() {
        let mut state = State::<board::TranspositionalEquivalence>::new(
            board::decode_board(r#"
.....____d.....
...._____d_....
..._________...
..___________..
.____d______d_.
___d_d_d____dd_
_d__d_____d_ddd
__d____O_______
ddd_____d______
Td__________dd_
.d__________d_.
..d_d_d______..
..._____d___...
...._______....
....._____.....
"#));
        state.do_action(&Action::HandleEndProposal(end::Decision::Decline));
        let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
        assert!(actions.is_empty());
    }

    #[test]
    fn troll_can_move_and_shove() {
        let state = State::<board::TranspositionalEquivalence>::new(
//             board::decode_board(r#"
// ....._____.....
// ....______d....
// ..._T___d__d...
// ..__________d..
// .d___d___d___d.
// d________d_____
// _______T_T____d
// _d___T_O_______
// _______T______d
// d___d_____T___d
// .d______T_____.
// ..d_T______dd..
// ..._____d___...
// ....d_____d....
// ....._d___.....
// "#)
            board::decode_board(r#"
....._____.....
....______d....
..._T___d__d...
..__________d..
.d___d___d___d.
d________d_____
_______T_T____d
_d_____O_______
______________d
d___d_________d
.d____________.
..d________dd..
..._____d___...
....d_____d....
....._d___.....
"#)
);
        let actions = {
            let mut v: Vec<Action> = state.role_actions(Role::Troll).collect();
            v.sort_by(::util::cmp_actions);
            v
        };
        let desired_actions = {
            let mut v = vec!(
                Action::ProposeEnd,
                // Troll at (2, 4).
                move_literal!((2, 4), (2, 3)),
                move_literal!((2, 4), (2, 5)),
                move_literal!((2, 4), (1, 4)),
                move_literal!((2, 4), (1, 5)),
                move_literal!((2, 4), (3, 4)),
                move_literal!((2, 4), (3, 5)),
                move_literal!((2, 4), (3, 3)),
                shove_literal!((2, 4), (3, 4), [(4, 5)]),
                shove_literal!((2, 4), (3, 5), [(4, 5)]),
                // Troll at (6, 7).
                move_literal!((6, 7), (6, 6)),
                move_literal!((6, 7), (6, 8)),
                move_literal!((6, 7), (5, 7)),
                move_literal!((6, 7), (5, 6)),
                move_literal!((6, 7), (5, 8)),
                move_literal!((6, 7), (7, 8)),
                move_literal!((6, 7), (7, 6)),
                shove_literal!((6, 7), (5, 6), [(4, 5)]),
                shove_literal!((6, 7), (6, 8), [(5, 9)]),
                shove_literal!((6, 7), (5, 8), [(4, 9), (5, 9)]),
                // Troll at (6, 9).
                move_literal!((6, 9), (6, 8)),
                move_literal!((6, 9), (6, 10)),
                move_literal!((6, 9), (7, 9)),
                move_literal!((6, 9), (5, 8)),
                move_literal!((6, 9), (5, 10)),
                move_literal!((6, 9), (7, 8)),
                move_literal!((6, 9), (7, 10)),
                shove_literal!((6, 9), (6, 8), [(5, 9)]),
                shove_literal!((6, 9), (6, 10), [(5, 9)]),
                shove_literal!((6, 9), (5, 8), [(5, 9), (4, 9)]),
                shove_literal!((6, 9), (5, 10), [(5, 9), (4, 9)]),
                // Troll at (7, 5).
            );
            v.sort_by(::util::cmp_actions);
            v
        };
        assert_eq!(actions, desired_actions);
    }

    #[test]
    fn dwarf_cant_move_illegally_ok() {
        let state = State::<board::TranspositionalEquivalence>::new(
            board::decode_board(r#"
.....dd_dd.....
....d_____d....
..._d______d...
..d_________d..
.d___________d.
d_____________d
d_____TTT_____d
______TOT______
d_____TT______d
d______T______d
.d___________d.
..d_________d..
...d_______d...
....d_____d....
.....dd_dd.....
"#));
        let actions: Vec<Action> = state.role_actions(Role::Dwarf).collect();
        assert!(!actions.contains(&move_literal!((9, 0), (9, 7))));
    }
}
