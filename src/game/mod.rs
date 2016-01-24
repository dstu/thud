mod actions;
pub mod board;

use std::hash::{Hash, Hasher, SipHasher};

pub use self::actions::Action;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    role: Role,
    name: String,
    marker: PlayerMarker,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn marker(&self) -> PlayerMarker {
        self.marker
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EndProposal {
    Neither,
    Single(PlayerMarker),
    Both,
}

impl EndProposal {
    fn advance(&mut self, player: PlayerMarker) {
        *self = match *self {
            EndProposal::Single(p) if p != player => EndProposal::Both,
            EndProposal::Neither => EndProposal::Single(player),
            _ => *self,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlayerMarker {
    One,
    Two,
}

impl PlayerMarker {
    pub fn index(self) -> usize {
        match self {
            PlayerMarker::One => 0,
            PlayerMarker::Two => 1,
        }
    }

    pub fn toggle(&mut self) {
        *self = match *self {
            PlayerMarker::One => PlayerMarker::Two,
            PlayerMarker::Two => PlayerMarker::One,
        }
    }
}

pub struct State {
    board: board::Cells,
    players: [Player; 2],
    active_player: PlayerMarker,
    end_proposal: EndProposal,
}

impl State {
    pub fn new(board: board::Cells, player1_name: String, player2_name: String) -> Self {
        State {
            board: board,
            players: [Player { role: Role::Dwarf, marker: PlayerMarker::One, name: player1_name, },
                      Player { role: Role::Troll, marker: PlayerMarker::Two, name: player2_name, },],
            active_player: PlayerMarker::One,
            end_proposal: EndProposal::Neither,
        }
    }

    pub fn cells(&self) -> &board::Cells {
        &self.board
    }

    pub fn new_default(player1_name: String, player2_name: String) -> Self {
        State::new(board::Cells::default(), player1_name, player2_name)
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.active_player.index()]
    }

    pub fn role_actions<'s>(&'s self, r: Role) -> actions::ActionIterator<'s> {
        self.board.role_actions(r)
    }

    pub fn position_actions<'s>(&'s self, position: board::Coordinate) -> actions::ActionIterator<'s> {
        self.board.position_actions(position)
    }

    pub fn toggle_active_player(&mut self) {
        self.active_player.toggle()
    }

    pub fn do_action(&mut self, a: &actions::Action) {
        match a {
            &actions::Action::Concede => self.end_proposal.advance(self.active_player),
            _ => self.board.do_action(a),
        }
        self.toggle_active_player();
    }

    pub fn terminated(&self) -> bool {
        self.end_proposal == EndProposal::Both
            || self.board.occupied_iter(Role::Dwarf).count() == 0
            || self.board.occupied_iter(Role::Troll).count() == 0
    }

    pub fn board(&self) -> &board::Cells {
        &self.board
    }

    pub fn player(&self, p: PlayerMarker) -> &Player {
        &self.players[p.index()]
    }

    pub fn end_proposal(&self) -> EndProposal {
        self.end_proposal
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        State {
            board: self.board.clone(),
            players: [self.players[0].clone(),
                      self.players[1].clone()],
            active_player: self.active_player,
            end_proposal: self.end_proposal,
        }
    }
}

impl PartialEq<State> for State {
    fn eq(&self, other: &State) -> bool {
        if self.active_player != other.active_player
            || self.end_proposal != other.end_proposal
            || self.players[0].role() != other.players[0].role() {
                return false
            }
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                for &c in [board::Coordinate::new(row, col),
                           board::Coordinate::new(14u8 - row, col),
                           board::Coordinate::new(row, 14u8 - col),
                           board::Coordinate::new(14u8 - row, 14u8 - col),
                           board::Coordinate::new(col, row),
                           board::Coordinate::new(14u8 - col, row),
                           board::Coordinate::new(col, 14u8 - row),
                           board::Coordinate::new(14u8 - col, 14u8 - row)].iter() {
                    if let Some(c) = c {
                        if self.board[c] != other.board[c] {
                            return false
                        }
                    }
                }
            }
        }
        true
    }
}

impl Eq for State { }

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hashers = [SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new(),
                           SipHasher::new()];
        for h in &mut hashers {
            self.active_player.hash(h);
            self.players[0].role().hash(h);
            self.players[1].role().hash(h);
            self.end_proposal.hash(h);
        }
        for row in 0u8..15u8 {
            for col in 0u8..15u8 {
                let mut i = 0;
                for &c in &[board::Coordinate::new(row, col),
                            board::Coordinate::new(14u8 - row, col),
                            board::Coordinate::new(row, 14u8 - col),
                            board::Coordinate::new(14u8 - row, 14u8 - col),
                            board::Coordinate::new(col, row),
                            board::Coordinate::new(14u8 - col, row),
                            board::Coordinate::new(col, 14u8 - row),
                            board::Coordinate::new(14u8 - col, 14u8 - row)] {
                    if let Some(c) = c {
                        self.board[c].hash(&mut hashers[i]);
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
                               hashers[7].finish()];
        (&mut hash_values).sort();
        for v in &hash_values {
            state.write_u64(*v);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ::actions;
    use ::board;
    use ::console_ui;
    use super::*;

    fn new_state() -> State {
        State::new(board::Cells::default(),
                   String::from_str("Player 1").ok().unwrap(),
                   String::from_str("Player 2").ok().unwrap())
    }

    #[test]
    fn null_move_equivalence() {
        assert!(new_state() == new_state())
    }

    #[test]
    fn simple_move_equivalence() {
        let mut s1 = new_state();
        s1.do_action(&actions::Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                            board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_state();
        s2.do_action(&actions::Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                            board::Coordinate::new(10, 3).unwrap()));
        assert!(s1 == s2);
    }

    #[test]
    fn simple_move_nonequivalence_1() {
        let mut s1 = new_state();
        s1.do_action(&actions::Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                            board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_state();
        s2.do_action(&actions::Action::Move(board::Coordinate::new(14, 9).unwrap(),
                                            board::Coordinate::new(6, 1).unwrap()));
        assert!(s1 != s2);
    }

    #[test]
    fn simple_move_nonequivalence_2() {
        let mut s1 = new_state();
        s1.do_action(&actions::Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                            board::Coordinate::new(8, 5).unwrap()));
        let mut s2 = new_state();
        s2.do_action(&actions::Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                            board::Coordinate::new(9, 5).unwrap()));
        assert!(s1 != s2);
    }
}
