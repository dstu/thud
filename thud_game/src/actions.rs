use super::coordinate::Coordinate;
use super::end;

use std::cmp::{Eq, PartialEq};
use std::fmt;

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
        write!(f, "Shove({:?}, {:?}, [", start, end)?;
        write!(f, "{:?}", captured[0])?;
        for i in 1..capture_count {
          write!(f, ", {:?}", captured[i as usize])?;
        }
        write!(f, "])")
      }
      &Action::ProposeEnd => write!(f, "ProposeEnd"),
      &Action::HandleEndProposal(d) => write!(f, "HandleEndProposal({:?})", d),
    }
  }
}

impl PartialEq<Action> for Action {
  fn eq(&self, rhs: &Action) -> bool {
    match (*self, *rhs) {
      (Action::Move(a_start, a_end), Action::Move(b_start, b_end)) => {
        a_start == b_start && a_end == b_end
      }
      (Action::Hurl(a_start, a_end), Action::Hurl(b_start, b_end)) => {
        a_start == b_start && a_end == b_end
      }
      (
        Action::Shove(a_start, a_end, a_capture_count, a_captures),
        Action::Shove(b_start, b_end, b_capture_count, b_captures),
      ) if a_start == b_start && a_end == b_end && a_capture_count == b_capture_count => {
        let mut a_captures_sorted = a_captures.clone();
        a_captures_sorted.sort();
        let mut b_captures_sorted = b_captures.clone();
        b_captures_sorted.sort();
        a_captures_sorted == b_captures_sorted
      }
      (Action::ProposeEnd, Action::ProposeEnd) => true,
      (Action::HandleEndProposal(d1), Action::HandleEndProposal(d2)) => d1 == d2,
      _ => false,
    }
  }
}

impl Eq for Action {}

#[macro_export]
macro_rules! move_literal {
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr)) => {
    $crate::actions::Action::Move(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
    )
  };
}

#[macro_export]
macro_rules! shove_literal {
    (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr)]) =>
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
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
        ($crate::actions::Action::Shove(
            $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
            $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
            7,
            [$crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_6_row, $capture_6_col),
             $crate::coordinate::Coordinate::new_unchecked($capture_7_row, $capture_7_col),]));
}

#[cfg(test)]
mod test {
  use crate::actions::Action;
  use crate::{board, end, Role};
  use crate::state::State;

  #[test]
  fn troll_can_move() {
    let state = State::<board::TranspositionalEquivalence>::new(board::decode_board(
      r#"
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
"#,
    ));
    let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
    assert!(!actions.is_empty());
    assert_eq!(Action::ProposeEnd, Action::ProposeEnd);
    assert_eq!(
      actions,
      vec!(
        Action::ProposeEnd,
        Action::Move(coordinate_literal!(5, 14), coordinate_literal!(4, 13)),
        Action::Shove(
          coordinate_literal!(5, 14),
          coordinate_literal!(4, 13),
          1,
          [
            coordinate_literal!(5, 13),
            coordinate_literal!(7, 7),
            coordinate_literal!(7, 7),
            coordinate_literal!(7, 7),
            coordinate_literal!(7, 7),
            coordinate_literal!(7, 7),
            coordinate_literal!(7, 7)
          ]
        )
      )
    );
  }

  #[test]
  fn troll_cant_move() {
    let mut state = State::<board::TranspositionalEquivalence>::new(board::decode_board(
      r#"
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
"#,
    ));
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
      board::decode_board(
        r#"
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
"#,
      ),
    );
    let actions = {
      let mut v: Vec<Action> = state.role_actions(Role::Troll).collect();
      v.sort_by(crate::util::cmp_actions);
      v
    };
    let desired_actions = {
      let mut v = vec![
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
      ];
      v.sort_by(crate::util::cmp_actions);
      v
    };
    assert_eq!(actions, desired_actions);
  }

  #[test]
  fn dwarf_cant_move_illegally_ok() {
    let state = State::<board::TranspositionalEquivalence>::new(board::decode_board(
      r#"
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
"#,
    ));
    let actions: Vec<Action> = state.role_actions(Role::Dwarf).collect();
    assert!(!actions.contains(&move_literal!((9, 0), (9, 7))));
  }
}
