//! Base interfaces defining a game whose state space can be searched with MCTS.

use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub trait State: Debug + Hash + Eq + Clone {
  type Action: Clone + Debug;
  type Payoff: Payoff<State = Self>;
  type PlayerId: Debug;

  fn active_player(&self) -> &Self::PlayerId;
  fn for_actions<F>(&self, f: F)
  where
    F: FnMut(Self::Action) -> bool;
  fn do_action(&mut self, action: &<Self as State>::Action);
  fn terminated(&self) -> bool;
}

pub trait Payoff: Debug + Add + AddAssign + Default {
  type State: State;
  type PlayerId: Debug;

  fn from_state(state: &Self::State) -> Option<Self>;
  fn visits(&self) -> u32;
  fn score(&self, player: &Self::PlayerId) -> f32;
}

pub trait Statistics: Clone + Debug + Default {
  type Payoff: Payoff;

  fn as_payoff(&self) -> Self::Payoff;
  fn increment(&self, payoff: &Self::Payoff);
}

pub trait Game: Debug + Clone {
  type Action: Clone + Debug;
  type PlayerId: Debug;
  type Payoff: Payoff<PlayerId = Self::PlayerId, State = Self::State>;
  type State: State<Action = Self::Action, Payoff = Self::Payoff, PlayerId = Self::PlayerId>;
  type Statistics: Statistics<Payoff = Self::Payoff>;
}
