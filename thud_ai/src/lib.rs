#![feature(const_fn)]

#[macro_use] extern crate log;
extern crate mcts;
extern crate rand;
extern crate syncbox;
extern crate thud_game;

use std::marker::PhantomData;

mod payoff;
mod state;
mod statistics;

pub use payoff::Payoff;
pub use state::State;
pub use statistics::Statistics;

pub struct Game<E> where E: thud_game::board::CellEquivalence {
    cell_equivalence: PhantomData<E>,
}

impl<E> Game<E> where E: thud_game::board::CellEquivalence {
    pub const fn new() -> Self {
        Game { cell_equivalence: PhantomData, }
    }
}

impl<E> mcts::Game for Game<E> where E: thud_game::board::CellEquivalence {
    type Action = thud_game::Action;
    type PlayerId = thud_game::Role;
    type Payoff = ::Payoff<E>;
    type State = ::State<E>;
    type Statistics = ::Statistics<E>;
}

pub const TRANSPOSITIONS_IGNORED: &'static Game<thud_game::board::TranspositionalEquivalence>
    = &Game::new();

pub const TRANSPOSITIONS_ALLOWED: &'static Game<thud_game::board::SimpleEquivalence> = &Game::new();
