#[macro_use] pub mod board;

mod actions;

use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

pub use self::actions::Action;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UnrecognizedRoleError(String);

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

impl FromStr for Role {
    type Err = UnrecognizedRoleError;

    fn from_str(s: &str) -> Result<Self, UnrecognizedRoleError> {
        match s.to_lowercase().as_str() {
            "dwarf" => Ok(Role::Dwarf),
            "troll" => Ok(Role::Troll),
            _ => Err(UnrecognizedRoleError(s.to_string())),
        }
    }
}

impl fmt::Display for UnrecognizedRoleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let UnrecognizedRoleError(ref s) = *self;
        write!(f, "Unrecognized role: {}", s)
    }
}

impl Error for UnrecognizedRoleError {
    fn description(&self) -> &str {
        "Unrecognized role"
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
    active_role: Role,
    end_proposal: EndProposal,
    equivalence_marker: PhantomData<E>,
}

impl<E> State<E> where E: board::CellEquivalence {
    pub fn new(board: board::Cells) -> Self {
        State {
            board: board,
            active_role: Role::Dwarf,
            end_proposal: EndProposal::Neither,
            equivalence_marker: PhantomData,
        }
    }

    pub fn cells(&self) -> &board::Cells {
        &self.board
    }

    pub fn active_role(&self) -> Role {
        self.active_role
    }

    pub fn role_actions<'s>(&'s self, r: Role) -> actions::ActionIterator<'s> {
        self.board.role_actions(r)
    }

    pub fn position_actions<'s>(&'s self, position: board::Coordinate) -> actions::ActionIterator<'s> {
        self.board.position_actions(position)
    }

    pub fn toggle_active_role(&mut self) {
        self.active_role = self.active_role.toggle()
    }

    pub fn do_action(&mut self, a: &actions::Action) {
        match a {
            // &actions::Action::Concede => self.end_proposal.advance(self.active_role),
            _ => self.board.do_action(a),
        }
        self.toggle_active_role();
    }

    pub fn terminated(&self) -> bool {
        self.end_proposal == EndProposal::Both
            || self.board.role_actions(Role::Dwarf).count() == 0
            || self.board.role_actions(Role::Troll).count() == 0
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
            active_role: self.active_role,
            end_proposal: self.end_proposal,
            equivalence_marker: PhantomData,
        }
    }
}

impl<E> PartialEq<State<E>> for State<E> where E: board::CellEquivalence {
    fn eq(&self, other: &State<E>) -> bool {
        if self.active_role != other.active_role
            || self.end_proposal != other.end_proposal {
                return false
            }
        <E as board::CellEquivalence>::boards_equal(&self.board, &other.board)
    }
}

impl<E> Eq for State<E> where E: board::CellEquivalence { }

impl<E> Hash for State<E> where E: board::CellEquivalence {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.active_role.hash(state);
        self.end_proposal.hash(state);
        <E as board::CellEquivalence>::hash_board(&self.board, state);
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    fn new_simple_state() -> State<board::SimpleEquivalence> {
        State::new(board::Cells::default())
    }

    fn new_untransposing_state() -> State<board::TranspositionalEquivalence> {
        State::new(board::Cells::default())
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
        s1.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
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
        s1.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
        check_hash_collision(s1, s2);
    }

    #[test]
    fn transposed_move_equivalence() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        assert!(s1 == s2);

        s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(5, 1)));
        assert!(s1 == s2);
    }

    #[test]
    fn transposed_move_hash_collision() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        check_hash_collision(s1, s2);

        s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(5, 0),
                                   coordinate_literal!(1, 5)));
        s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(5, 1)));
        check_hash_collision(s1, s2);
    }

    #[test]
    fn simple_move_nonequivalence_1() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(coordinate_literal!(14, 9),
                                   coordinate_literal!(6, 1)));
        assert!(s1 != s2);
    }

    #[test]
    fn simple_move_nonequivalence_2() {
        let mut s1 = new_simple_state();
        s1.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(8, 5)));
        let mut s2 = new_simple_state();
        s2.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(9, 5)));
        assert!(s1 != s2);
    }

    #[test]
    fn transposed_move_nonequivalence_1() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(2, 3),
                                   coordinate_literal!(10, 3)));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(14, 9),
                                   coordinate_literal!(6, 1)));
        assert!(s1 != s2);
    }

    #[test]
    fn transposed_move_nonequivalence_2() {
        let mut s1 = new_untransposing_state();
        s1.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(8, 5)));
        let mut s2 = new_untransposing_state();
        s2.do_action(&Action::Move(coordinate_literal!(0, 5),
                                   coordinate_literal!(9, 5)));
        assert!(s1 != s2);
    }
}
