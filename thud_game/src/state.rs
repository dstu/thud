use crate::actions::Action;
use crate::board::{CellEquivalence, Cells};
use crate::coordinate::Coordinate;
use crate::end;
use crate::Role;
use r4::iterate;

use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct State {
  board: Cells,
  active_role: Role,
  proposed_terminate: bool,
  terminate_decision: Option<end::Decision>,
}

impl State {
  pub fn new(board: Cells) -> Self {
    State {
      board: board,
      active_role: Role::Dwarf,
      proposed_terminate: false,
      terminate_decision: None,
    }
  }

  pub fn cells(&self) -> &Cells {
    &self.board
  }

  pub fn active_role(&self) -> &Role {
    &self.active_role
  }

  pub fn actions<'s>(&'s self) -> impl Iterator<Item = Action> + 's {
    self.role_actions(*self.active_role())
  }

  pub fn role_actions<'s>(&'s self, r: Role) -> impl Iterator<Item = Action> + 's {
    let must_handle_end_proposal = self.proposed_terminate && self.terminate_decision.is_none();
    let handle_end_proposal = iterate![if must_handle_end_proposal;
                     yield Action::HandleEndProposal(end::Decision::Accept);
                     yield Action::HandleEndProposal(end::Decision::Decline)];
    let regular_moves = iterate![if !must_handle_end_proposal;
                     for action in self.board.role_actions(r, self.terminate_decision.is_none());
                     yield action];
    handle_end_proposal.chain(regular_moves)
  }

  pub fn position_actions<'s>(&'s self, position: Coordinate) -> impl Iterator<Item = Action> + 's {
    self.board.position_actions(position)
  }

  pub fn toggle_active_role(&mut self) {
    self.active_role = self.active_role.toggle()
  }

  // pub fn set_from_convolved(&mut self, convolved: &Self) {
  //   use super::board::CellEquivalence;

  //   let mut convolutions = Convolution::all().iter();
  //   loop {
  //     match convolutions.next() {
  //       None => panic!("no match"),
  //       Some(v) => {
  //         let mut scratch_cells = Cells::new();
  //         for (coordinate, content) in convolved.board.cells_iter() {
  //           scratch_cells[v.convolve(coordinate)] = content;
  //         }
  //         if board::SimpleEquivalence::boards_equal(&convolved.board, &scratch_cells) {
  //           for (coordinate, content) in convolved.board.cells_iter() {
  //             self.board[v.inverse(coordinate)] = content;
  //           }
  //           break;
  //         }
  //       }
  //     }
  //   }
  //   self.active_role = convolved.active_role;
  //   self.proposed_terminate = convolved.proposed_terminate;
  //   self.terminate_decision = convolved.terminate_decision;
  // }

  pub fn do_action(&mut self, a: &Action) {
    match a {
      &Action::ProposeEnd => self.proposed_terminate = true,
      &Action::HandleEndProposal(d) => self.terminate_decision = Some(d),
      _ => {
        self.proposed_terminate = false;
        self.terminate_decision = None;
        self.board.do_action(a)
      }
    }
    self.toggle_active_role();
  }

  pub fn terminated(&self) -> bool {
    self.terminate_decision == Some(end::Decision::Accept)
      || self.board.role_actions(Role::Dwarf, false).count() == 0
      || self.board.role_actions(Role::Troll, false).count() == 0
  }

  pub fn board(&self) -> &Cells {
    &self.board
  }

  pub fn opponent_proposed_end(&self) -> bool {
    self.proposed_terminate
  }
}

/// Wraps around a state and makes it possible to store it in a hash-based
/// container.
#[derive(Clone, Debug)]
pub struct AddressableState<E: CellEquivalence> {
  pub state: State,
  marker: PhantomData<E>,
}

impl<E: CellEquivalence> AddressableState<E> {
  pub fn new(state: State) -> Self {
    AddressableState {
      state: state,
      marker: PhantomData,
    }
  }
}

impl<E: CellEquivalence> Hash for AddressableState<E> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    <E as CellEquivalence>::hash_board(&self.state.board, state);
    self.state.active_role.hash(state);
    self.state.proposed_terminate.hash(state);
    self.state.terminate_decision.hash(state);
  }
}

impl <E: CellEquivalence> PartialEq for AddressableState<E> {
  fn eq(&self, other: &Self) -> bool {
    <E as CellEquivalence>::boards_equal(&self.state.board, &other.state.board) &&
      self.state.active_role == other.state.active_role &&
      self.state.proposed_terminate == other.state.proposed_terminate &&
      self.state.terminate_decision == other.state.terminate_decision
  }
}

impl<E: CellEquivalence> Eq for AddressableState<E> {}

impl<E: CellEquivalence> From<State> for AddressableState<E> {
  fn from(state: State) -> Self {
    AddressableState::new(state)
  }
}

impl<E: CellEquivalence> Borrow<State> for AddressableState<E> {
  fn borrow(&self) -> &State {
    &self.state
  }
}

impl<E: CellEquivalence> BorrowMut<State> for AddressableState<E> {
  fn borrow_mut(&mut self) -> &mut State {
    &mut self.state
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::actions::Action;
  use crate::board::{self, Cells};
  use crate::end;
  use std::collections::HashMap;

  fn new_state() -> State {
    State::new(Cells::default())
  }

  fn new_simple_state() -> AddressableState<board::SimpleEquivalence> {
    new_state().into()
  }

  fn new_untransposing_state() -> AddressableState<board::TranspositionalEquivalence> {
    new_state().into()
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
    s1.state.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.state.do_action(&move_literal!((2, 3), (10, 3)));
    assert!(s1 == s2);
  }

  fn check_hash_collision<E: board::CellEquivalence>(s1: AddressableState<E>, s2: AddressableState<E>) {
    let mut table = HashMap::new();
    assert!(table.is_empty());
    table.insert(s1, true);
    assert_eq!(1, table.len());
    assert!(table.contains_key(&s2));
  }

  #[test]
  fn simple_move_hash_collision() {
    let mut s1 = new_simple_state();
    s1.state.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.state.do_action(&move_literal!((2, 3), (10, 3)));
    check_hash_collision(s1, s2);
  }

  #[test]
  fn transposed_move_equivalence() {
    let mut s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((5, 0), (1, 5)));
    let mut s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((5, 0), (1, 5)));
    assert!(s1 == s2);

    s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((5, 0), (1, 5)));
    s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((0, 5), (5, 1)));
    assert!(s1 == s2);
  }

  #[test]
  fn transposed_move_hash_collision() {
    let mut s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((5, 0), (1, 5)));
    let mut s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((5, 0), (1, 5)));
    check_hash_collision(s1, s2);

    s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((5, 0), (1, 5)));
    s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((0, 5), (5, 1)));
    check_hash_collision(s1, s2);
  }

  #[test]
  fn simple_move_nonequivalence_1() {
    let mut s1 = new_simple_state();
    s1.state.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.state.do_action(&move_literal!((14, 9), (6, 1)));
    assert!(s1 != s2);
  }

  #[test]
  fn simple_move_nonequivalence_2() {
    let mut s1 = new_simple_state();
    s1.state.do_action(&move_literal!((0, 5), (8, 5)));
    let mut s2 = new_simple_state();
    s2.state.do_action(&move_literal!((0, 5), (9, 5)));
    assert!(s1 != s2);
  }

  #[test]
  fn transposed_move_nonequivalence_1() {
    let mut s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((14, 9), (6, 1)));
    assert!(s1 != s2);
  }

  #[test]
  fn transposed_move_nonequivalence_2() {
    let mut s1 = new_untransposing_state();
    s1.state.do_action(&move_literal!((0, 5), (8, 5)));
    let mut s2 = new_untransposing_state();
    s2.state.do_action(&move_literal!((0, 5), (9, 5)));
    assert!(s1 != s2);
  }

  #[test]
  fn propose_end_ok() {
    let state = new_simple_state();
    assert!(!state.state.proposed_terminate);
    assert_eq!(state.state.terminate_decision, None);
    let available_actions: Vec<Action> = state.state.actions().collect();
    assert!(available_actions.contains(&Action::ProposeEnd));
  }

  #[test]
  fn no_propose_end_ok() {
    let mut state = new_simple_state();
    assert!(!state.state.proposed_terminate);
    state.state.do_action(&Action::ProposeEnd);
    assert!(state.state.proposed_terminate);
    let available_actions: Vec<Action> = state.state.actions().collect();
    assert_eq!(
      available_actions,
      vec!(
        Action::HandleEndProposal(end::Decision::Accept),
        Action::HandleEndProposal(end::Decision::Decline)
      )
    );
    assert!(!state.state.terminated());
  }

  #[test]
  fn accept_terminate_ok() {
    let mut state = new_simple_state();
    state.state.do_action(&Action::ProposeEnd);
    state.state.do_action(&Action::HandleEndProposal(end::Decision::Accept));
    assert!(state.state.terminated());
  }

  #[test]
  fn decline_terminate_ok() {
    let mut state = new_simple_state();
    state.state.do_action(&Action::ProposeEnd);
    state.state.do_action(&Action::HandleEndProposal(end::Decision::Decline));
    assert!(!state.state.terminated());
    let mut available_actions: Vec<Action> = state.state.actions().collect();
    assert!(!available_actions.contains(&Action::ProposeEnd));

    state.state.do_action(&available_actions[0]);
    available_actions = state.state.actions().collect();
    assert!(available_actions.contains(&Action::ProposeEnd));
  }
}
