use crate::Role;
use crate::actions::Action;
use crate::state::State;
use mcts;

mod payoff;
mod statistics;

pub use self::payoff::Payoff;
pub use self::statistics::Statistics;

#[derive(Clone, Debug)]
pub struct Game {}

impl mcts::State for State {
  type Action = crate::actions::Action;
  type Payoff = Payoff;
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

impl mcts::Game for Game {
  type Action = Action;
  type PlayerId = Role;
  type Payoff = Payoff;
  type State = crate::state::State;
  type Statistics = Statistics;
}
