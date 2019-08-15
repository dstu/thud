//! Interface for deriving payoffs from game state.

use crate::game::{Game, State};
use crate::SearchSettings;
use log::trace;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

use rand::Rng;
use rand::seq::IteratorRandom;

pub trait Simulator: for<'a> From<&'a SearchSettings> {
  type Error: Error;

  fn simulate<G: Game, R: Rng>(
    &self,
    state: &G::State,
    rng: &mut R,
  ) -> Result<G::Payoff, Self::Error>;
}

pub struct RandomSimulator {
  simulation_count: u32,
}

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
  fn from(settings: &'a SearchSettings) -> Self {
    RandomSimulator {
      simulation_count: settings.simulation_count,
    }
  }
}

impl Simulator for RandomSimulator {
  type Error = RandomSimulatorError;

  fn simulate<G: Game, R: Rng>(
    &self,
    state: &G::State,
    rng: &mut R,
  ) -> Result<G::Payoff, Self::Error> {
    let mut payoff = G::Payoff::default();
    if let Some(p) = G::payoff_of(state) {
      for _ in 0..self.simulation_count {
        payoff += &p;
      }
    } else {
      for _ in 0..self.simulation_count {
        let mut simulation_state = state.clone();
        loop {
          match simulation_state.actions().choose(rng) {
            Some(a) => {
              trace!("doing action: {:?}", a);
              simulation_state.do_action(&a);
              trace!("updated state: {:?}", state);
            }
            None => return Err(RandomSimulatorError::DeadEnd),
          }
          if let Some(p) = G::payoff_of(&simulation_state) {
            payoff += &p;
            break;
          }
        }
      }
    }
    Ok(payoff)
  }
}
