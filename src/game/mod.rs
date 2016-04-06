mod actions;
pub mod board;

use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub use self::actions::Action;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

impl Role {
    pub fn index(self) -> usize {
        match self {
            Role::Dwarf => 0,
            Role::Troll => 1,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Role::Dwarf => Role::Troll,
            Role::Troll => Role::Dwarf,
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EndProposal {
    Neither,
    Single(Role),
    Both,
}

impl EndProposal {
    fn advance(&mut self, role: Role) {
        *self = match *self {
            EndProposal::Single(r) if r != role => EndProposal::Both,
            EndProposal::Neither => EndProposal::Single(role),
            _ => *self,
        }
    }
}

pub struct State<E> where E: board::CellEquivalence {
    board: board::Cells,
    players: [Player; 2],
    active_player: Role,
    end_proposal: EndProposal,
    equivalence_marker: PhantomData<E>,
}

impl<E> State<E> where E: board::CellEquivalence {
    pub fn new(board: board::Cells, player1_name: String, player2_name: String) -> Self {
        State {
            board: board,
            players: [Player { role: Role::Dwarf, name: player1_name, },
                      Player { role: Role::Troll, name: player2_name, },],
            active_player: Role::Dwarf,
            end_proposal: EndProposal::Neither,
            equivalence_marker: PhantomData,
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
        self.active_player = self.active_player.toggle()
    }

    pub fn do_action(&mut self, a: &actions::Action) {
        match a {
            // &actions::Action::Concede => self.end_proposal.advance(self.active_player),
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

    pub fn end_proposal(&self) -> EndProposal {
        self.end_proposal
    }
}

impl<E> Clone for State<E> where E: board::CellEquivalence {
    fn clone(&self) -> Self {
        State {
            board: self.board.clone(),
            players: [self.players[0].clone(),
                      self.players[1].clone()],
            active_player: self.active_player,
            end_proposal: self.end_proposal,
            equivalence_marker: PhantomData,
        }
    }
}

impl<E> PartialEq<State<E>> for State<E> where E: board::CellEquivalence {
    fn eq(&self, other: &State<E>) -> bool {
        if self.active_player != other.active_player
            || self.end_proposal != other.end_proposal
            || self.players[0].role() != other.players[0].role() {
                return false
            }
        <E as board::CellEquivalence>::boards_equal(&self.board, &other.board)
    }
}

impl<E> Eq for State<E> where E: board::CellEquivalence { }

impl<E> Hash for State<E> where E: board::CellEquivalence {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.active_player.hash(state);
        self.players[0].role().hash(state);
        self.players[1].role().hash(state);
        self.end_proposal.hash(state);
        <E as board::CellEquivalence>::hash_board(&self.board, state);
    }
}



#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::collections::HashMap;

    use super::*;

    fn new_simple_state() -> State<board::SimpleEquivalence> {
        State::new(board::Cells::default(),
                   String::from_str("Player 1").ok().unwrap(),
                   String::from_str("Player 2").ok().unwrap())
    }

    fn new_untransposing_state() -> State<board::TranspositionalEquivalence> {
        State::new(board::Cells::default(),
                   String::from_str("Player 1").ok().unwrap(),
                   String::from_str("Player 2").ok().unwrap())
    }

    #[test]
    fn null_move_equivalence() {
        assert!(new_simple_state() == new_simple_state());
        assert!(new_untransposing_state() == new_untransposing_state());
    }

    #[test]
    fn simple_null_move_hash_collision() {
        let mut table = HashMap::new();
        let s1 = new_simple_state();
        assert!(table.is_empty());
        table.insert(s1, true);
        assert_eq!(1, table.len());
        assert!(table.contains_key(&new_simple_state()));
    }

    #[test]
    fn transposed_null_move_hash_collision() {
        let mut table = HashMap::new();
        let s1 = new_untransposing_state();
        assert!(table.is_empty());
        table.insert(s1, true);
        assert_eq!(1, table.len());
        assert!(table.contains_key(&new_untransposing_state()));
    }

    #[test]
    fn simple_move_equivalence() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        assert!(s1 == s2);
    }

    fn check_hash_collision<E>(s1: State<E>, s2: State<E>) where E: ::game::board::CellEquivalence {
        let mut table = HashMap::new();
        assert!(table.is_empty());
        table.insert(s1, true);
        assert_eq!(1, table.len());
        assert!(table.contains_key(&s2));
    }

    #[test]
    fn simple_move_hash_collision() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        check_hash_collision(s1, s2);
    }

    #[test]
    fn transposed_move_equivalence() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        assert!(s1 == s2);

        s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(5, 1).unwrap()));
        assert!(s1 == s2);
    }

    #[test]
    fn transposed_move_hash_collision() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        check_hash_collision(s1, s2);

        s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(5, 0).unwrap(),
                                   board::Coordinate::new(1, 5).unwrap()));
        s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(5, 1).unwrap()));
        check_hash_collision(s1, s2);
    }

    #[test]
    fn simple_move_nonequivalence_1() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(board::Coordinate::new(14, 9).unwrap(),
                                   board::Coordinate::new(6, 1).unwrap()));
        assert!(s1 != s2);
    }

    #[test]
    fn simple_move_nonequivalence_2() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(8, 5).unwrap()));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(9, 5).unwrap()));
        assert!(s1 != s2);
    }

    #[test]
    fn transposed_move_nonequivalence_1() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(2, 3).unwrap(),
                                   board::Coordinate::new(10, 3).unwrap()));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(14, 9).unwrap(),
                                   board::Coordinate::new(6, 1).unwrap()));
        assert!(s1 != s2);
    }

    #[test]
    fn transposed_move_nonequivalence_2() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(8, 5).unwrap()));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(board::Coordinate::new(0, 5).unwrap(),
                                   board::Coordinate::new(9, 5).unwrap()));
        assert!(s1 != s2);
    }
}
