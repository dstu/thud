#[macro_use] extern crate log;
extern crate mcts;
extern crate rand;
extern crate syncbox;
extern crate thud_game;

mod payoff;
mod state;
mod statistics;

pub use payoff::Payoff;
pub use state::State;
pub use statistics::Statistics;
use std::marker::PhantomData;


pub struct Game<E> where E: thud_game::board::CellEquivalence {
    cell_equivalence: PhantomData<E>,
}

impl<E> mcts::Game for Game<E> where E: thud_game::board::CellEquivalence {
    type Action = thud_game::Action;
    type PlayerId = thud_game::Role;
    type Payoff = ::Payoff<E>;
    type State = ::State<E>;
    type Statistics = ::Statistics<E>;
}

pub mod allow_transpositions {
    pub type Game = ::Game<::thud_game::board::TranspositionalEquivalence>;
    pub type Payoff = ::Payoff<::thud_game::board::TranspositionalEquivalence>;
    pub type State = ::State<::thud_game::board::TranspositionalEquivalence>;
    pub type Statistics = ::Statistics<::thud_game::board::TranspositionalEquivalence>;
}

pub mod deconvolve_transpositions {
    pub type Game = ::Game<::thud_game::board::TranspositionalEquivalence>;
    pub type Payoff = ::Payoff<::thud_game::board::TranspositionalEquivalence>;
    pub type State = ::State<::thud_game::board::TranspositionalEquivalence>;
    pub type Statistics = ::Statistics<::thud_game::board::TranspositionalEquivalence>;
}
