//! Single-threaded Monte Carlo tree search on directed acyclic graphs.

pub mod backprop;
pub mod game;
pub mod graph;
pub mod rollout;
pub mod simulation;
pub mod ucb;

use self::backprop::BackpropSelector;
use self::rollout::RolloutSelector;
use self::simulation::Simulator;

use self::game::{Game, Payoff, State, Statistics};
use self::graph::{EdgeData, VertexData};

use std::convert::From;
use std::result::Result;

use rand::Rng;

/// Wraps a decision made by the UCB rollout policy.
///
/// This is distinct from other types in the `ucb` module to provide a
/// representation of the decision of UCB rollout that is not bound to the
/// lifetime of a `search_graph` structure.
#[derive(Clone, Debug)]
pub enum UcbValue {
  /// Select a game state because it has not yet been explored (and so no
  /// finite UCB policy value is available).
  Select,
  /// Select a game state with the given UCB policy value.
  Value(f64),
}

impl<'a> From<&ucb::UcbSuccess<'a>> for UcbValue {
  fn from(success: &ucb::UcbSuccess<'a>) -> Self {
    match *success {
      ucb::UcbSuccess::Select(_) => UcbValue::Select,
      ucb::UcbSuccess::Value(_, v) => UcbValue::Value(v),
    }
  }
}

/// Statistics for a specific game action.
///
/// This type is used for reporting summary statistics for the next decision to
/// make after executing search.
#[derive(Clone, Debug)]
pub struct ActionStatistics<G: Game> {
  /// The action.
  pub action: G::Action,
  /// The action's expected payoff.
  pub payoff: G::Payoff,
  /// The result of UCB rollout for that action (used for debugging MCTS with
  /// a UCB rollout policy).
  pub ucb: Result<UcbValue, ucb::UcbError>,
}

/// Creates a new search graph suitable for Monte Carlo tree search through the
/// state space of the game `G`.
pub fn new_search_graph<G: Game>() -> search_graph::Graph<G::State, VertexData, EdgeData<G>> {
  search_graph::Graph::<G::State, VertexData, EdgeData<G>>::new()
}

/// Settings for a Monte Carlo tree search.
#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
  /// The number of simulations to run when estimating payout of a new game state.
  pub simulation_count: u32,
  /// The exploration bias term to use for the UCB policy.
  pub explore_bias: f64,
}

///
pub struct RolloutPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'a>,
}

impl<'a, R: Rng, G: Game> RolloutPhase<'a, R, G> {
  pub fn initialize(
    rng: R,
    settings: SearchSettings,
    root_state: G::State,
    mut graph: search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  ) -> Self {
    let root_node = match graph.find_node(&root_state) {
      Some(n) => n,
      None => graph.append_node(root_state.clone(), VertexData::default()),
    };
    RolloutPhase {
      rng,
      settings,
      graph,
      root_node,
    }
  }

  pub fn rollout<S: RolloutSelector<G, R>>(
    mut self,
  ) -> Result<ScoringPhase<'a, R, G>, rollout::RolloutError<'a, S::Error>> {
    rollout::rollout(
      &self.graph,
      self.root_node,
      S::from(&self.settings),
      &mut self.rng,
    )
    .map(|node| ScoringPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: node,
    })
  }
}

pub struct ScoringPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'a>,
  rollout_node: search_graph::view::NodeRef<'a>,
}

impl<'a, R: Rng, G: Game> ScoringPhase<'a, R, G> {
  pub fn score<S: Simulator<G, R>>(mut self) -> Result<BackpropPhase<'a, R, G>, S::Error> {
    let payoff = match G::Payoff::from_state(self.graph.node_state(self.rollout_node)) {
      Some(p) => p,
      None => S::from(&self.settings).simulate(
        self.graph.node_state(self.rollout_node).clone(),
        &mut self.rng,
      )?,
    };
    Ok(BackpropPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: self.rollout_node,
      payoff,
    })
  }
}

pub struct BackpropPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'a>,
  rollout_node: search_graph::view::NodeRef<'a>,
  payoff: G::Payoff,
}

impl<'a, R: Rng, G: Game> BackpropPhase<'a, R, G> {
  pub fn backprop<S: BackpropSelector<'a, G, R>>(mut self) -> ExpandPhase<'a, R, G> {
    backprop::backprop(
      &self.graph,
      self.rollout_node,
      &self.payoff,
      &S::from(&self.settings),
      &mut self.rng,
    );
    ExpandPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
      rollout_node: self.rollout_node,
    }
  }
}

pub struct ExpandPhase<'a, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'a>,
  rollout_node: search_graph::view::NodeRef<'a>,
}

impl<'a, R: Rng, G: Game> ExpandPhase<'a, R, G> {
  pub fn expand(mut self) -> RolloutPhase<'a, R, G> {
    if !self.graph.node_data(self.rollout_node).mark_expanded() {
      self
        .graph
        .node_state(self.rollout_node)
        .clone()
        .for_actions(|action| {
          let mut child_state = self.graph.node_state(self.rollout_node).clone();
          child_state.do_action(&action);
          let child = match self.graph.find_node(&child_state) {
            Some(n) => n,
            None => self
              .graph
              .append_node(child_state.clone(), Default::default()),
          };
          self
            .graph
            .append_edge(self.rollout_node, child, EdgeData::new(action));
          true
        });
    }
    RolloutPhase {
      rng: self.rng,
      settings: self.settings,
      graph: self.graph,
      root_node: self.root_node,
    }
  }
}
