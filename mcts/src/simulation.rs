use super::{Game, Payoff, State};

use std::error::Error;
use std::result::Result;

use ::rand::Rng;

pub trait Simulator<G, R> where G: Game, R: Rng {
    type Error: Error;

    fn simulate(&self, settings: &SearchSettings, state: G::State, rng: &mut R)
                -> Result<G::Payoff, Self::Error>;
}
