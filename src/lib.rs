use std::iter::Iterator;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Piece {
    Thudstone,
    Dwarf,
    Troll,
}

// For now, assume that we're on a standard Thud grid.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Coordinate(u8);

const ROW_LENGTHS: [u8; 15] = [5, 7, 9, 11, 13, 15, 15, 15, 15, 15, 13, 11, 9, 7, 5];
const ROW_OFFSETS: [u8; 15] = [0, 5, 12, 21, 32, 45, 60, 75, 90, 105, 120, 133, 144, 153, 160];

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

    pub fn up_left(self) -> Option<Self> {
        self.up().and_then(|n| n.left())
    }

    pub fn up_right(self) -> Option<Self> {
        self.up().and_then(|n| n.right())
    }

    pub fn down_left(self) -> Option<Self> {
        self.down().and_then(|n| n.left())
    }

    pub fn down_right(self) -> Option<Self> {
        self.down().and_then(|n| n.right())
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
    pub fn new() -> Self {
        let mut board = BoardState { spaces: [None; 165], };
        // Place Dwarfs.
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
        board.set_piece_at(c.up_left().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.up_right().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.down_left().unwrap(), Some(Piece::Troll));
        board.set_piece_at(c.down_right().unwrap(), Some(Piece::Troll));
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Player {
    Dwarf,
    Troll,
}

pub struct GameState {
    current_player: Player,
    actions: Vec<Action>,
    board: BoardState,
}

macro_rules! build_dwarf_move_actions {
    ($board: expr, $start: expr, $mutator_expr: expr, $result: ident) => (
        {
            let mutator = $mutator_expr;
            let mut end = mutator($start);
            loop {
                match end {
                    Some(e) if $board.piece_at(e).is_none() => {
                        $result.push(Action::Move($start, e));
                        end = mutator(e);
                    },
                    _ => break,
                }
            }
        })
}

macro_rules! build_dwarf_hurl_actions {
    ($board: expr, $start: expr, $forward_expr: expr, $backward_expr: expr, $result: ident) => (
        {
            let forward = $forward_expr;
            let backward = $backward_expr;
            let mut end = forward($start);
            let mut line_start = $start;
            loop {
                match end {
                    Some(e) if $board.piece_at(e) == Some(Piece::Troll) => {  // Land on a troll.
                        $result.push(Action::Hurl($start, e));
                        break
                    },
                    Some(e) if $board.piece_at(e).is_none() => {  // No obstacles.
                        match backward(line_start) {
                            Some(s) if $board.piece_at(s) == Some(Piece::Dwarf) => {
                                // More dwarfs behind, so continue search.
                                line_start = s;
                                end = forward(e);
                            },
                            _ => break,  // No more dwarfs behind.
                        }
                    },
                    _ => break,  // Ran off of end of board or hit a dwarf or the Thudstone.
                }
            }
        })
}

macro_rules! build_troll_move_actions {
    ($board: expr, $start: expr, $mutator_expr: expr, $result: ident) => (
        {
            let mutator = $mutator_expr;
            match mutator($start) {
                Some(e) if $board.piece_at(e).is_none() =>
                    $result.push(Action::Move($start, e)),  // Nothing in the way.
                _ => (),  // Obstacle.
            }
        })
}

macro_rules! build_troll_shove_actions {
    ($board: expr, $start: expr, $forward_expr: expr, $backward_expr: expr, $result: ident) => (
        {
            let forward = $forward_expr;
            let backward = $backward_expr;
            let mut end = forward($start);
            let mut line_start = $start;
            loop {
                match end {
                    Some(e) if $board.piece_at(e) == None => {
                        if (e.up().and_then(|c| $board.piece_at(c))
                            .or_else(|| e.down().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.left().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.right().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.up_left().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.up_right().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.down_left().and_then(|c| $board.piece_at(c)))
                            .or_else(|| e.down_right().and_then(|c| $board.piece_at(c))))
                            == Some(Piece::Dwarf) {
                                // At least one dwarf is adjacent to this space, so a
                                // shove that lands here will capture.
                                $result.push(Action::Shove($start, e));
                                match backward(line_start) {
                                    Some(s) if $board.piece_at(s) == Some(Piece::Troll) => {
                                        // More trolls behind, so continue search.
                                        line_start = s;
                                        end = forward(e);
                                    },
                                    _ => break,  // No more trolls behind.
                                }
                            }
                    },
                    _ => break,  // Ran off of end of board or hit an occupied square.
                }
            }
        })
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            current_player: Player::Dwarf,
            actions: vec![],
            board: BoardState::new(),
        }
    }

    pub fn player_actions(&self, p: Player) -> Vec<Action> {
        let mut result = vec![];
        match p {
            Player::Dwarf =>
                for i in 0u8..165u8 {
                    let start = Coordinate::new(i);
                    if let Some(Piece::Dwarf) = self.board.piece_at(start) {
                        // Move.
                        build_dwarf_move_actions![
                            self.board, start, |c: Coordinate| c.up(), result];
                        build_dwarf_move_actions![
                            self.board, start, |c: Coordinate| c.down(), result];
                        build_dwarf_move_actions![
                            self.board, start, |c: Coordinate| c.left(), result];
                        build_dwarf_move_actions![
                            self.board, start, |c: Coordinate| c.right(), result];
                        build_dwarf_move_actions![
                            self.board, start,
                            |c: Coordinate| c.up_left(), result];
                        build_dwarf_move_actions![
                            self.board, start,
                            |c: Coordinate| c.up_right(), result];
                        build_dwarf_move_actions![
                            self.board, start,
                            |c: Coordinate| c.down_left(), result];
                        build_dwarf_move_actions![
                            self.board, start,
                            |c: Coordinate| c.down_right(), result];
                        // Hurl.
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.up(), |c: Coordinate| c.down(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.down(), |c: Coordinate| c.up(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.left(), |c: Coordinate| c.right(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.right(), |c: Coordinate| c.left(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.up_left(), |c: Coordinate| c.down_right(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.up_right(), |c: Coordinate| c.down_left(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.down_left(), |c: Coordinate| c.up_right(), result];
                        build_dwarf_hurl_actions![
                            self.board, start,
                            |c: Coordinate| c.down_right(), |c: Coordinate| c.up_left(), result];
                    }
                },
            Player::Troll =>
                for i in 0u8..165u8 {
                    let start = Coordinate::new(i);
                    if let Some(Piece::Troll) = self.board.piece_at(start) {
                        // Move.
                        build_troll_move_actions![
                            self.board, start, |c: Coordinate| c.up(), result];
                        build_troll_move_actions![
                            self.board, start, |c: Coordinate| c.down(), result];
                        build_troll_move_actions![
                            self.board, start, |c: Coordinate| c.left(), result];
                        build_troll_move_actions![
                            self.board, start, |c: Coordinate| c.right(), result];
                        build_troll_move_actions![
                            self.board, start,
                            |c: Coordinate| c.up_left(), result];
                        build_troll_move_actions![
                            self.board, start,
                            |c: Coordinate| c.up_right(), result];
                        build_troll_move_actions![
                            self.board, start,
                            |c: Coordinate| c.down_left(), result];
                        build_troll_move_actions![
                            self.board, start,
                            |c: Coordinate| c.down_right(), result];
                        // Shove.
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.up(), |c: Coordinate| c.down(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.down(), |c: Coordinate| c.up(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.left(), |c: Coordinate| c.right(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.right(), |c: Coordinate| c.left(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.up_left(),
                            |c: Coordinate| c.down_right(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.up_right(),
                            |c: Coordinate| c.down_left(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.down_left(),
                            |c: Coordinate| c.up_right(), result];
                        build_troll_shove_actions![
                            self.board, start,
                            |c: Coordinate| c.down_right(),
                            |c: Coordinate| c.up_left(), result];
                    }
                },
            }
        result
    }

    pub fn player(&self) -> Player {
        self.current_player
    }

    pub fn advance_player(&mut self) {
        self.current_player = match self.current_player {
            Player::Dwarf => Player::Troll,
            Player::Troll => Player::Dwarf,
        }
    }

    fn remove_adjacent_dwarves(&mut self, end: Coordinate) {
        for adjacency in [end.up(), end.down(), end.left(), end.right(),
                          end.up_left(), end.up_right(),
                          end.down_left(), end.down_right()].iter() {
            if let Some((e, Some(Piece::Dwarf))) = adjacency.map(|c| (c, self.board.piece_at(c))) {
                self.board.set_piece_at(e, None);
            }
        }
    }

    pub fn take_action(&mut self, a: Action) {
        // TODO: validation.
        self.actions.push(a);
        match a {
            Action::Move(start, end) => {
                let p = self.board.piece_at(start);
                self.board.set_piece_at(end, p);
                self.board.set_piece_at(start, None);
                if self.current_player == Player::Troll {
                    self.remove_adjacent_dwarves(end);
                }
            },
            Action::Hurl(start, end) => {
                let p = self.board.piece_at(start);
                self.board.set_piece_at(end, p);
                self.board.set_piece_at(start, None);
            },
            Action::Shove(start, end) => {
                let p = self.board.piece_at(start);
                self.board.set_piece_at(end, p);
                self.board.set_piece_at(start, None);
                self.remove_adjacent_dwarves(end);
            },
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Move(Coordinate, Coordinate),
    Hurl(Coordinate, Coordinate),
    Shove(Coordinate, Coordinate),
}
