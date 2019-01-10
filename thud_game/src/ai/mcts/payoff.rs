use crate::{board, Role};
use mcts;

use std::default::Default;
use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Payoff {
  pub weight: u32,
  pub values: [u32; 2],
}

impl Payoff {
  pub fn new(weight: u32, dwarf: u32, troll: u32) -> Self {
    let mut values = [0, 0];
    values[Role::Dwarf.index()] = dwarf;
    values[Role::Troll.index()] = troll;
    Payoff {
      weight: weight,
      values: values,
    }
  }
}

impl Default for Payoff {
  fn default() -> Self {
    Payoff {
      weight: 0,
      values: [0, 0],
    }
  }
}

impl Add for Payoff {
  type Output = Payoff;

  fn add(self, other: Payoff) -> Payoff {
    Payoff {
      weight: self.weight + other.weight,
      values: [
        self.values[0] + other.values[0],
        self.values[1] + other.values[1],
      ],
    }
  }
}

impl AddAssign for Payoff {
  fn add_assign(&mut self, other: Payoff) {
    self.weight += other.weight;
    self.values[0] += other.values[0];
    self.values[1] += other.values[1];
  }
}

impl fmt::Debug for Payoff {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "[{}, {}]@{}",
      self.values[0], self.values[1], self.weight
    )
  }
}

fn role_payoff(r: Role) -> u32 {
  match r {
    Role::Dwarf => 1,
    Role::Troll => 4,
  }
}

impl mcts::Payoff for Payoff {
  type State = crate::state::State;
  type PlayerId = Role;

  fn from_state(state: &Self::State) -> Option<Payoff> {
    if state.terminated() {
      let mut payoff = Payoff::default();
      payoff.weight = 1;
      let mut i = state.cells().cells_iter();
      loop {
        match i.next() {
          Some((_, board::Content::Occupied(t))) => {
            if let Some(r) = t.role() {
              payoff.values[r.index()] += role_payoff(r)
            }
          }
          None => break,
          _ => (),
        }
      }
      Some(payoff)
    } else {
      None
    }
  }

  fn visits(&self) -> u32 {
    self.weight
  }

  fn score(&self, player: &Role) -> f32 {
    self.values[player.index()] as f32
  }
}
