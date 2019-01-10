use crate::actions::Action;
use crate::board::{CellEquivalence, Cells};
use crate::coordinate::Coordinate;
use crate::end;
use crate::Role;
use r4::iterate;

use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct State {
  board: Cells,
  active_role: Role,
  proposed_terminate: bool,
  terminate_decision: Option<end::Decision>,
  equivalence_class: &'static dyn CellEquivalence,
}

impl State {
  pub fn new(board: Cells, equivalence_class: &'static CellEquivalence) -> Self {
    State {
      board: board,
      active_role: Role::Dwarf,
      proposed_terminate: false,
      terminate_decision: None,
      equivalence_class: equivalence_class,
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

impl Clone for State {
  fn clone(&self) -> Self {
    State {
      board: self.board.clone(),
      active_role: self.active_role,
      proposed_terminate: self.proposed_terminate,
      terminate_decision: self.terminate_decision,
      equivalence_class: self.equivalence_class,
    }
  }
}

impl Hash for State {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (self.equivalence_class.hash_board(&self.board)).hash(state);
    self.active_role.hash(state);
    self.proposed_terminate.hash(state);
    self.terminate_decision.hash(state);
  }
}

impl PartialEq for State {
  fn eq(&self, other: &Self) -> bool {
    self.equivalence_class.boards_equal(&self.board, &other.board) &&
      self.active_role == other.active_role &&
      self.proposed_terminate == other.proposed_terminate &&
      self.terminate_decision == other.terminate_decision
  }
}

impl Eq for State {}

#[cfg(test)]
mod test {
  use super::*;
  use crate::actions::Action;
  use crate::board::{self, Cells};
  use crate::end;
  use std::collections::HashMap;

  fn new_simple_state() -> State {
    State::new(Cells::default(), &board::SIMPLE_EQUIVALENCE)
  }

  fn new_untransposing_state() -> State {
    State::new(Cells::default(), &board::TRANSPOSITIONAL_EQUIVALENCE)
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
    s1.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.do_action(&move_literal!((2, 3), (10, 3)));
    assert!(s1 == s2);
  }

  fn check_hash_collision(s1: State, s2: State) {
    let mut table = HashMap::new();
    assert!(table.is_empty());
    table.insert(s1, true);
    assert_eq!(1, table.len());
    assert!(table.contains_key(&s2));
  }

  #[test]
  fn simple_move_hash_collision() {
    let mut s1 = new_simple_state();
    s1.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.do_action(&move_literal!((2, 3), (10, 3)));
    check_hash_collision(s1, s2);
  }

  #[test]
  fn transposed_move_equivalence() {
    let mut s1 = new_untransposing_state();
    s1.do_action(&move_literal!((5, 0), (1, 5)));
    let mut s2 = new_untransposing_state();
    s2.do_action(&move_literal!((5, 0), (1, 5)));
    assert!(s1 == s2);

    s1 = new_untransposing_state();
    s1.do_action(&move_literal!((5, 0), (1, 5)));
    s2 = new_untransposing_state();
    s2.do_action(&move_literal!((0, 5), (5, 1)));
    assert!(s1 == s2);
  }

  #[test]
  fn transposed_move_hash_collision() {
    let mut s1 = new_untransposing_state();
    s1.do_action(&move_literal!((5, 0), (1, 5)));
    let mut s2 = new_untransposing_state();
    s2.do_action(&move_literal!((5, 0), (1, 5)));
    check_hash_collision(s1, s2);

    s1 = new_untransposing_state();
    s1.do_action(&move_literal!((5, 0), (1, 5)));
    s2 = new_untransposing_state();
    s2.do_action(&move_literal!((0, 5), (5, 1)));
    check_hash_collision(s1, s2);
  }

  #[test]
  fn simple_move_nonequivalence_1() {
    let mut s1 = new_simple_state();
    s1.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_simple_state();
    s2.do_action(&move_literal!((14, 9), (6, 1)));
    assert!(s1 != s2);
  }

  #[test]
  fn simple_move_nonequivalence_2() {
    let mut s1 = new_simple_state();
    s1.do_action(&move_literal!((0, 5), (8, 5)));
    let mut s2 = new_simple_state();
    s2.do_action(&move_literal!((0, 5), (9, 5)));
    assert!(s1 != s2);
  }

  #[test]
  fn transposed_move_nonequivalence_1() {
    let mut s1 = new_untransposing_state();
    s1.do_action(&move_literal!((2, 3), (10, 3)));
    let mut s2 = new_untransposing_state();
    s2.do_action(&move_literal!((14, 9), (6, 1)));
    assert!(s1 != s2);
  }

  #[test]
  fn transposed_move_nonequivalence_2() {
    let mut s1 = new_untransposing_state();
    s1.do_action(&move_literal!((0, 5), (8, 5)));
    let mut s2 = new_untransposing_state();
    s2.do_action(&move_literal!((0, 5), (9, 5)));
    assert!(s1 != s2);
  }

  #[test]
  fn propose_end_ok() {
    let state = new_simple_state();
    assert!(!state.proposed_terminate);
    assert_eq!(state.terminate_decision, None);
    let available_actions: Vec<Action> = state.actions().collect();
    assert!(available_actions.contains(&Action::ProposeEnd));
  }

  #[test]
  fn no_propose_end_ok() {
    let mut state = new_simple_state();
    assert!(!state.proposed_terminate);
    state.do_action(&Action::ProposeEnd);
    assert!(state.proposed_terminate);
    let available_actions: Vec<Action> = state.actions().collect();
    assert_eq!(
      available_actions,
      vec!(
        Action::HandleEndProposal(end::Decision::Accept),
        Action::HandleEndProposal(end::Decision::Decline)
      )
    );
    assert!(!state.terminated());
  }

  #[test]
  fn accept_terminate_ok() {
    let mut state = new_simple_state();
    state.do_action(&Action::ProposeEnd);
    state.do_action(&Action::HandleEndProposal(end::Decision::Accept));
    assert!(state.terminated());
  }

  #[test]
  fn decline_terminate_ok() {
    let mut state = new_simple_state();
    state.do_action(&Action::ProposeEnd);
    state.do_action(&Action::HandleEndProposal(end::Decision::Decline));
    assert!(!state.terminated());
    let mut available_actions: Vec<Action> = state.actions().collect();
    assert!(!available_actions.contains(&Action::ProposeEnd));

    state.do_action(&available_actions[0]);
    available_actions = state.actions().collect();
    assert!(available_actions.contains(&Action::ProposeEnd));
  }
}
