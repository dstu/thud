//! Interface for deriving payoffs from game state.

use crate::game::{Game, LoopControl, State};
use crate::SearchSettings;
use log::trace;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

use rand::Rng;
use rand::seq::SliceRandom;

pub trait Simulator: for<'a> From<&'a SearchSettings> {
  type Error: Error;

  fn simulate<G: Game, R: Rng>(
    &self,
    state: &G::State,
    rng: &mut R,
  ) -> Result<G::Payoff, Self::Error>;
}

pub struct RandomSimulator {}

#[derive(Debug)]
pub enum RandomSimulatorError {
  DeadEnd,
}

impl fmt::Display for RandomSimulatorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Error for RandomSimulatorError {
  fn description(&self) -> &'static str {
    "game state has no valid actions"
  }
}
  

impl<'a> From<&'a SearchSettings> for RandomSimulator {
  fn from(_: &'a SearchSettings) -> Self {
    RandomSimulator {}
  }
}

impl Simulator for RandomSimulator {
  type Error = RandomSimulatorError;

  fn simulate<G: Game, R: Rng>(
    &self,
    state: &G::State,
    rng: &mut R,
  ) -> Result<G::Payoff, Self::Error> {
    if let Some(p) = G::payoff_of(state) {
      return Ok(p);
    }
    let mut state = state.clone();
    loop {
      match G::payoff_of(&state) {
        Some(p) => return Ok(p),
        None => {
          let mut actions: Vec<G::Action> = Vec::new();
          state.for_actions(|a| {
            actions.push(a);
            LoopControl::Continue
          });
          match actions.choose(rng) {
            Some(a) => {
              trace!("doing action: {:?}", a);
              state.do_action(&a);
              trace!("updated state: {:?}", state);
            }
            None => return Err(RandomSimulatorError::DeadEnd),
          }
        }
      }
    }
  }
}
