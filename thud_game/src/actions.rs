use crate::coordinate::Coordinate;
use crate::end;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{Eq, PartialEq};
use std::str::FromStr;
use std::{error, fmt};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionParseError {
  /// Basic tokenization of move failed.
  PatternMismatch,
  /// Player indicator was invalid.
  InvalidPlayer,
  /// Move contained an invalid coordinate.
  InvalidCoordinate(String),
  /// Capture component of a move is not valid.
  InvalidCapture,
}

fn parse_coordinate(s: &str) -> Result<Coordinate, ActionParseError> {
  if s.len() < 2 || s.len() > 3 {
    return Err(ActionParseError::PatternMismatch);
  }
  let mut i = s.chars();
  let c1 = i.next().unwrap();
  let row = match c1 {
    'A' => 0,
    'B' => 1,
    'C' => 2,
    'D' => 3,
    'E' => 4,
    'F' => 5,
    'G' => 6,
    'H' => 7,
    'J' => 8,
    'K' => 9,
    'L' => 10,
    'M' => 11,
    'N' => 12,
    'O' => 13,
    'P' => 14,
    _ => return Err(ActionParseError::InvalidCoordinate(s.into())),
  };
  let column = match i.as_str().parse::<u8>() {
    Ok(n) if n > 0 && n < 16 => n - 1,
    Ok(_) | Err(_) => return Err(ActionParseError::InvalidCoordinate(s.into())),
  };
  match Coordinate::new(row, column) {
    Some(c) => Ok(c),
    None => Err(ActionParseError::InvalidCoordinate(s.into())),
  }
}

fn parse_captures(s: &str) -> Result<Vec<Coordinate>, ActionParseError> {
  let mut captures: Vec<Coordinate> = Vec::with_capacity(7);
  for capture_str in s.split("x").skip(1) {
    captures.push(parse_coordinate(capture_str)?);
  }
  Ok(captures)
}

impl FromStr for Action {
  type Err = ActionParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    lazy_static! {
      static ref RE: Regex =
        Regex::new(r#"(?x)
^(?:
   # Propose end of game.
   (?P<end>end)
   # Confirm end of game.
  |(?P<confirm>confirm)
   # Refuse end of game.
  |(?P<refuse>refuse)
   # Actual action.
  |# Player taking action.
   (?P<player>[dt])
   # Source coordinate.
   \x20(?P<source>...?)
   # Target coordinate.
     \-(?P<target>...?)
   # Optional captures.
   (?P<capture>(?:x...?)+)?
)$"#).unwrap();
    }

    let captures = match RE.captures(s.trim()) {
      None => return Err(ActionParseError::PatternMismatch),
      Some(c) => c,
    };

    if let Some(_) = captures.name("end") {
      return Ok(Action::ProposeEnd);
    }
    if let Some(_) = captures.name("confirm") {
      return Ok(Action::HandleEndProposal(end::Decision::Accept));
    }
    if let Some(_) = captures.name("refuse") {
      return Ok(Action::HandleEndProposal(end::Decision::Decline));
    }

    let player = match captures.name("player") {
      Some(s) if s.as_str() == "d" => crate::Role::Dwarf,
      Some(s) if s.as_str() == "t" => crate::Role::Troll,
      _ => return Err(ActionParseError::InvalidPlayer),
    };

    let from_coordinate = match captures.name("source").map(|s| parse_coordinate(s.as_str())) {
      Some(Ok(c)) => c,
      Some(Err(e)) => return Err(e),
      None => return Err(ActionParseError::PatternMismatch),
    };

    let to_coordinate = match captures.name("target").map(|s| parse_coordinate(s.as_str())) {
      Some(Ok(c)) => c,
      Some(Err(e)) => return Err(e),
      None => return Err(ActionParseError::PatternMismatch),
    };

    let capture_coordinates = match captures.name("capture").map(|s| parse_captures(s.as_str())) {
      None => Vec::new(),
      Some(Ok(v)) => v,
      Some(Err(e)) => return Err(e),
    };
    if player == crate::Role::Dwarf && capture_coordinates.len() > 1 {
      return Err(ActionParseError::InvalidCapture);
    }
    if capture_coordinates.len() >= 7 {
      return Err(ActionParseError::InvalidCapture);
    }
    let mut capture_coordinates_array = [Coordinate::new_unchecked(6, 6); 7];
    for (i, c) in capture_coordinates.iter().enumerate() {
      capture_coordinates_array[i] = *c;
    }

    let action = match player {
      crate::Role::Dwarf => {
        if capture_coordinates.is_empty() {
          Action::Move(from_coordinate, to_coordinate)
        } else {
          Action::Hurl(from_coordinate, to_coordinate)
        }
      }
      crate::Role::Troll => {
        if capture_coordinates.is_empty() {
          Action::Move(from_coordinate, to_coordinate)
        } else {
          Action::Shove(
            from_coordinate,
            to_coordinate,
            capture_coordinates.len() as u8,
            capture_coordinates_array,
          )
        }
      }
    };
    Ok(action)
  }
}

impl error::Error for ActionParseError {}

impl fmt::Display for ActionParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

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
     [($capture_1_row: expr, $capture_1_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      1,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      2,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      3,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      4,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      5,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr),
      ($capture_6_row: expr, $capture_6_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      6,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_6_row, $capture_6_col),
        $crate::coordinate::Coordinate::new_unchecked(7, 7),
      ],
    )
  };
  (($start_row: expr, $start_col: expr), ($end_row: expr, $end_col: expr),
     [($capture_1_row: expr, $capture_1_col: expr),
      ($capture_2_row: expr, $capture_2_col: expr),
      ($capture_3_row: expr, $capture_3_col: expr),
      ($capture_4_row: expr, $capture_4_col: expr),
      ($capture_5_row: expr, $capture_5_col: expr),
      ($capture_6_row: expr, $capture_6_col: expr),
      ($capture_7_row: expr, $capture_7_col: expr)]) => {
    $crate::actions::Action::Shove(
      $crate::coordinate::Coordinate::new_unchecked($start_row, $start_col),
      $crate::coordinate::Coordinate::new_unchecked($end_row, $end_col),
      7,
      [
        $crate::coordinate::Coordinate::new_unchecked($capture_1_row, $capture_1_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_2_row, $capture_2_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_3_row, $capture_3_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_4_row, $capture_4_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_5_row, $capture_5_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_6_row, $capture_6_col),
        $crate::coordinate::Coordinate::new_unchecked($capture_7_row, $capture_7_col),
      ],
    )
  };
}

#[cfg(test)]
mod test {
  use crate::actions::Action;
  use crate::state::State;
  use crate::{board, end, Role};

  #[test]
  fn troll_can_move() {
    let state = State::new(
      board::decode_board(
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
      ),
      &board::TRANSPOSITIONAL_EQUIVALENCE,
    );
    let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
    assert!(!actions.is_empty());
    assert_eq!(Action::ProposeEnd, Action::ProposeEnd);
    assert_eq!(
      actions,
      vec!(
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
        ),
        Action::ProposeEnd,
      )
    );
  }

  #[test]
  fn troll_cant_move() {
    let mut state = State::new(
      board::decode_board(
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
      ),
      &board::TRANSPOSITIONAL_EQUIVALENCE,
    );
    state.do_action(&Action::HandleEndProposal(end::Decision::Decline));
    let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
    assert_eq!(actions, vec!());
  }

  #[test]
  fn troll_can_move_and_shove() {
    let state = State::new(
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
      &board::TRANSPOSITIONAL_EQUIVALENCE,
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
      ];
      v.sort_by(crate::util::cmp_actions);
      v
    };
    assert_eq!(actions, desired_actions);
  }

  #[test]
  fn dwarf_cant_move_illegally_ok() {
    let state = State::new(
      board::decode_board(
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
      ),
      &board::TRANSPOSITIONAL_EQUIVALENCE,
    );
    let actions: Vec<Action> = state.role_actions(Role::Dwarf).collect();
    assert!(!actions.contains(&move_literal!((9, 0), (9, 7))));
  }

  #[test]
  fn troll_doesnt_have_illegal_move() {
    let state = State::new(
      board::decode_board(
    r#"
.....dd_dd.....
....d_____d....
...d_______d...
..d_________d..
.d___d_______d.
d______T______d
d_____TT______d
______TOT______
d_____TTT_____d
______________d
.dd__________d.
..d_________d..
...d_______d...
....d_____d....
.....d__dd.....
"#,
      ),
      &board::TRANSPOSITIONAL_EQUIVALENCE,
    );
    let actions: Vec<Action> = state.role_actions(Role::Troll).collect();
    println!("{:?}", actions);
    assert!(!actions.contains(&move_literal!((9, 7), (10, 8))));
  }
}
