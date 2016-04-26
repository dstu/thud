use ::mcts;
use ::thud_game;

use std::fmt;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct State<E> where E: thud_game::board::CellEquivalence {
    pub wrapped: thud_game::state::State<E>,
}

impl<E> State<E> where E: thud_game::board::CellEquivalence {
    pub fn new(cells: thud_game::board::Cells) -> Self {
        State { wrapped: thud_game::state::State::<E>::new(cells), }
    }
}

impl<E> fmt::Debug for State<E>
    where E: thud_game::board::CellEquivalence {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self.wrapped.cells(), f)
        }
}

impl<E> mcts::State for State<E> where E: thud_game::board::CellEquivalence {
    type Action = thud_game::Action;
    type Payoff = super::Payoff<E>;
    type PlayerId = thud_game::Role;

    fn active_player(&self) -> &thud_game::Role {
        &self.wrapped.active_role()
    }

    fn for_actions<F>(&self, mut f: F) where F: FnMut(thud_game::Action) -> bool {
        let mut actions = self.wrapped.actions();
        let mut done = false;
        while !done {
            done = match actions.next() {
                None => true,
                Some(a) => f(a),
            }
        }
    }

    fn do_action(&mut self, action: &thud_game::Action) {
        self.wrapped.do_action(action);
    }

    fn terminated(&self) -> bool {
        self.wrapped.terminated()
    }
}
