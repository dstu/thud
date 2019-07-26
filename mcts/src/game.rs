//! Base interfaces defining a game whose state space can be searched with MCTS.

use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;

pub trait State: Debug + Hash + Eq + Clone {
  type Action: Clone + Debug;
  type Payoff: Debug;
  type PlayerId: Debug;

  fn active_player(&self) -> &Self::PlayerId;
  fn for_actions<F>(&self, f: F)
  where
    F: FnMut(Self::Action) -> bool;
  fn do_action(&mut self, action: &<Self as State>::Action);
}

pub trait Statistics<S: State, P>: Clone + Debug + Default {
  fn increment(&self, payoff: &P);
  fn visits(&self) -> u32;
  fn score(&self, player: &S::PlayerId) -> f32;
}

pub trait PayoffFn<S, P>: Debug {
  fn payoff_of(state: &S) -> Option<P>;
}

pub trait Game: Debug {
  type Action: Clone + Debug;
  type PlayerId: Debug;
  type Payoff: Debug;
  type State: State<Action = Self::Action, Payoff = Self::Payoff, PlayerId = Self::PlayerId>;
  type Statistics: Statistics<Self::State, Self::Payoff>;

  fn payoff_of(state: &Self::State) -> Option<Self::Payoff>;
}
