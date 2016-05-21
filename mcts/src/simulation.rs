//! Interface for deriving payoffs from game state.

use super::{Game, SearchSettings};

use std::convert::From;
use std::error::Error;
use std::result::Result;

use ::rand::Rng;

pub trait Simulator<G, R>: From<SearchSettings> where G: Game, R: Rng {
    type Error: Error;

    fn simulate(&self, state: G::State, rng: &mut R) -> Result<G::Payoff, Self::Error>;
}
