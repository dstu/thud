//! Interface for deriving payoffs from game state.

use crate::game::{Game, State};
use crate::SearchSettings;
use log::trace;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::result::Result;

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
  thread_pool: ThreadPool,
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

impl RandomSimulator {
  /// Runs a single simulation.
  fn run_simulation<G: Game, R: Rng>(
    mut state: G::State,
    mut rng: R,
  ) -> Result<G::Payoff, RandomSimulatorError> {
    loop {
      if let Some(p) = G::payoff_of(&state) {
        return Ok(p);
      }
      match state.actions().choose(&mut rng) {
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

impl<'a> From<&'a SearchSettings> for RandomSimulator {
  fn from(settings: &'a SearchSettings) -> Self {
    RandomSimulator {
      simulation_count: settings.simulation_count,
      thread_pool: ThreadPoolBuilder::new()
        .num_threads(settings.simulation_thread_limit as usize)
        .thread_name(|n| format!("random-simulator-thread-{}", n))
        .build()
        .unwrap(),
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
    if let Some(p) = G::payoff_of(state) {
      let mut payoff = G::Payoff::default();
      for _ in 0..self.simulation_count {
        payoff += &p;
      }
      Ok(payoff)
    } else {
      let parameters = (0..self.simulation_count)
        .map(|_| {
          let mut seed = [0u8; 32];
          rng.fill_bytes(&mut seed);
          (state.clone(), Pcg64::from_seed(seed))
        })
        .collect::<Vec<(G::State, Pcg64)>>();

      self.thread_pool.install(move || {
        parameters
          .into_par_iter()
          .try_fold(
            || G::Payoff::default(),
            |mut acc, params| {
              let (state, rng) = params;
              match RandomSimulator::run_simulation::<G, _>(state, rng) {
                Err(e) => Err(e),
                Ok(p) => {
                  acc += &p;
                  Ok(acc)
                }
              }
            },
          )
          .try_reduce(
            || G::Payoff::default(),
            |mut acc, result| {
              acc += &result;
              Ok(acc)
            },
          )
      })
    }
  }
}
