use crate::{board, Role};
use crate::actions::Action;
use crate::state::State;
use mcts;

mod payoff;
mod statistics;

pub use self::payoff::Payoff;
pub use self::statistics::Statistics;

use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Game<E>
where
  E: board::CellEquivalence,
{
  cell_equivalence: PhantomData<E>,
}

impl<E> ::mcts::State for State<E>
where
  E: board::CellEquivalence,
{
  type Action = crate::actions::Action;
  type Payoff = Payoff<E>;
  type PlayerId = Role;

  fn active_player(&self) -> &Role {
    &self.active_role()
  }

  fn for_actions<F>(&self, mut f: F)
  where
    F: FnMut(Action) -> bool,
  {
    let mut actions = self.actions();
    let mut done = false;
    while !done {
      done = match actions.next() {
        None => true,
        Some(a) => f(a),
      }
    }
  }

  fn do_action(&mut self, action: &Action) {
    self.do_action(action);
  }

  fn terminated(&self) -> bool {
    self.terminated()
  }
}

impl<E> mcts::Game for Game<E>
where
  E: board::CellEquivalence,
{
  type Action = Action;
  type PlayerId = Role;
  type Payoff = Payoff<E>;
  type State = crate::state::State<E>;
  type Statistics = Statistics<E>;
}

pub mod allow_transpositions {
  pub type Game = super::Game<crate::board::SimpleEquivalence>;
  pub type Payoff = super::Payoff<crate::board::SimpleEquivalence>;
  pub type State = crate::state::State<crate::board::SimpleEquivalence>;
  pub type Statistics = super::Statistics<crate::board::SimpleEquivalence>;
}

pub mod deconvolve_transpositions {
  pub type Game = super::Game<crate::board::TranspositionalEquivalence>;
  pub type Payoff = super::Payoff<crate::board::TranspositionalEquivalence>;
  pub type State = crate::state::State<crate::board::TranspositionalEquivalence>;
  pub type Statistics = super::Statistics<crate::board::TranspositionalEquivalence>;
}
