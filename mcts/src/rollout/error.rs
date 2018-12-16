//! Error type for MCTS rollout.

use crate::game::Game;
use crate::graph::{EdgeData, VertexData};

use std::convert::From;
use std::error::Error;
use std::fmt;

use search_graph::nav::Edge;

pub enum RolloutError<'a, G, E>
where
  G: 'a + Game,
  E: Error,
{
  Cycle(Vec<Edge<'a, G::State, VertexData, EdgeData<G>>>),
  Selector(E),
}

impl<'a, G: 'a + Game, E: Error> fmt::Debug for RolloutError<'a, G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({:?})", e),
    }
  }
}

impl<'a, G: 'a + Game, E: Error> fmt::Display for RolloutError<'a, G, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RolloutError::Cycle(_) => write!(f, "Cycle in path"),
      RolloutError::Selector(ref e) => write!(f, "Selector error ({})", e),
    }
  }
}

impl<'a, G: 'a + Game, E: Error> Error for RolloutError<'a, G, E> {
  fn description(&self) -> &str {
    match *self {
      RolloutError::Cycle(_) => "Cycle",
      RolloutError::Selector(_) => "Selector error",
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      RolloutError::Selector(ref e) => Some(e),
      _ => None,
    }
  }
}

impl<'a, G: 'a + Game, E: Error> From<E> for RolloutError<'a, G, E> {
  fn from(e: E) -> Self {
    RolloutError::Selector(e)
  }
}
