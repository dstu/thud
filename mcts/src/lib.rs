//! Single-threaded Monte Carlo tree search on directed acyclic graphs.

pub mod backprop;
pub mod game;
pub mod graph;
pub mod rollout;
pub mod simulation;
pub mod statistics;
#[cfg(test)]
pub(crate) mod tictactoe;
pub mod ucb;

use self::backprop::BackpropSelector;
use self::rollout::RolloutSelector;
use self::simulation::Simulator;

use self::game::{Game, State};
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

/// Settings for a round of Monte Carlo tree search.
#[derive(Clone, Copy, Debug)]
pub struct SearchSettings {
  /// The number of simulations to run when estimating payout of a new game state.
  pub simulation_count: u32,
  /// The exploration bias term to use for the UCB policy.
  pub explore_bias: f64,
}

/// Recursively traverses the search graph to find a game state from which to
/// perform payoff estimates.
pub struct RolloutPhase<'a, 'id, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'id>,
}

impl<'a, 'id, R: Rng, G: Game> RolloutPhase<'a, 'id, R, G> {
  pub fn initialize(
    rng: R,
    settings: SearchSettings,
    root_state: G::State,
    mut graph: search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
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

  pub fn rollout<S: RolloutSelector>(
    mut self,
  ) -> Result<ScoringPhase<'a, 'id, R, G>, rollout::RolloutError<'id, S::Error>> {
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

  pub fn root_node(&self) -> search_graph::view::NodeRef<'id> {
    self.root_node
  }
}

/// Computes an estimate of the score for a game state selected during rollout.
pub struct ScoringPhase<'a, 'id, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'id>,
  rollout_node: search_graph::view::NodeRef<'id>,
}

impl<'a, 'id, R: Rng, G: Game> ScoringPhase<'a, 'id, R, G> {
  pub fn root_node(&self) -> search_graph::view::NodeRef<'id> {
    self.root_node
  }

  pub fn rollout_node(&self) -> search_graph::view::NodeRef<'id> {
    self.rollout_node
  }

  pub fn score<S: Simulator>(mut self) -> Result<BackpropPhase<'a, 'id, R, G>, S::Error> {
    let payoff = match G::payoff_of(self.graph.node_state(self.rollout_node())) {
      Some(p) => p,
      None => {
        let simulator = S::from(&self.settings);
        simulator.simulate::<G, R>(self.graph.node_state(self.rollout_node()), &mut self.rng)?
      }
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

/// Performs backprop of a known payoff from a node selected during rollout.
///
/// The strategy for finding the game-state statistics to update during backprop
/// is determined by `BackpropSelector`.
pub struct BackpropPhase<'a, 'id, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'id>,
  rollout_node: search_graph::view::NodeRef<'id>,
  payoff: G::Payoff,
}

impl<'a, 'id, R: Rng, G: Game> BackpropPhase<'a, 'id, R, G> {
  pub fn backprop<S: BackpropSelector<'id>>(mut self) -> ExpandPhase<'a, 'id, R, G> {
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

/// Expands a node in the search graph.
///
/// This is done by playing out each of the legal moves at the node's game
/// state, adding them to the graph if they don't already exist, and then
/// creating an edge from the original node to the node for the resulting game
/// state.
pub struct ExpandPhase<'a, 'id, R: Rng, G: Game> {
  rng: R,
  settings: SearchSettings,
  graph: search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  root_node: search_graph::view::NodeRef<'id>,
  rollout_node: search_graph::view::NodeRef<'id>,
}

impl<'a, 'id, R: Rng, G: Game> ExpandPhase<'a, 'id, R, G> {
  pub fn expand(mut self) -> RolloutPhase<'a, 'id, R, G> {
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

#[cfg(test)]
mod test {
  use crate::graph::{EdgeData, VertexData};
  use crate::ucb;
  use crate::{
    backprop, simulation, tictactoe, BackpropPhase, ExpandPhase, RolloutPhase, ScoringPhase,
    SearchSettings,
  };
  use rand::SeedableRng;
  use rand_pcg;

  fn default_rng() -> rand_pcg::Pcg64 {
    rand_pcg::Pcg64::from_seed([0; 32])
  }

  fn default_game_state() -> tictactoe::State {
    Default::default()
  }

  fn default_settings() -> SearchSettings {
    SearchSettings {
      simulation_count: 1,
      explore_bias: 1.0,
    }
  }

  type Graph = search_graph::Graph<tictactoe::State, VertexData, EdgeData<tictactoe::ScoredGame>>;

  #[test]
  fn rollout_init() {
    let mut graph = Graph::new();

    search_graph::view::of_graph(&mut graph, |view| {
      RolloutPhase::initialize(
        default_rng(),
        default_settings(),
        default_game_state(),
        view,
      );
    });

    let node = graph.find_node(&default_game_state());
    assert!(node.is_some());
    let node = node.unwrap();
    assert!(node.is_leaf());
    assert!(node.is_root());
    assert_eq!(default_game_state(), *node.get_label());
  }

  #[test]
  fn integration_test() {
    let mut graph = Graph::new();

    search_graph::view::of_graph(&mut graph, |view| {
      // Rollout.
      let rollout_phase = RolloutPhase::initialize(
        default_rng(),
        default_settings(),
        default_game_state(),
        view,
      );
      let rollout_target = crate::rollout::rollout(
        &rollout_phase.graph,
        rollout_phase.root_node,
        ucb::Rollout::from(&default_settings()),
        &mut default_rng(),
      )
      .unwrap();
      assert_eq!(rollout_phase.root_node(), rollout_target);
      assert_eq!(
        tictactoe::State::default(),
        *rollout_phase.graph.node_state(rollout_target)
      );

      // Scoring.
      let scoring_phase: ScoringPhase<'_, '_, _, tictactoe::ScoredGame> =
        rollout_phase.rollout::<ucb::Rollout>().unwrap();
      assert_eq!(scoring_phase.rollout_node(), rollout_target);
      assert_eq!(scoring_phase.root_node(), rollout_target);

      // Backprop.
      let backprop_phase: BackpropPhase<'_, '_, _, tictactoe::ScoredGame> = scoring_phase
        .score::<simulation::RandomSimulator>()
        .unwrap();

      // Expand.
      let expand_phase: ExpandPhase<'_, '_, _, tictactoe::ScoredGame> =
        backprop_phase.backprop::<backprop::FirstParentSelector>();
      expand_phase.expand();
    });

    {
      let node = graph.find_node(&Default::default()).unwrap();
      assert!(node.is_root());
      assert!(!node.is_leaf());
      assert_eq!(9, node.get_child_list().len());
      for child in node.get_child_list().iter() {
        assert_eq!(0, child.get_data().statistics.visits());
        assert_eq!(
          0,
          child
            .get_data()
            .statistics
            .score(crate::statistics::two_player::Player::One)
        );
        assert_eq!(
          0,
          child
            .get_data()
            .statistics
            .score(crate::statistics::two_player::Player::Two)
        );
        assert!(child.get_target().is_leaf());
        assert!(!child.get_target().is_root());
      }
    }

    search_graph::view::of_graph(&mut graph, |view| {
      RolloutPhase::initialize(
        default_rng(),
        default_settings(),
        default_game_state(),
        view,
      )
      .rollout::<ucb::Rollout>()
      .unwrap()
      .score::<simulation::RandomSimulator>()
      .unwrap()
      .backprop::<backprop::FirstParentSelector>()
      .expand();
    });

    {
      let node = graph.find_node(&Default::default()).unwrap();
      assert!(node.is_root());
      assert!(!node.is_leaf());
      assert_eq!(9, node.get_child_list().len());
      for (i, child) in node.get_child_list().iter().enumerate() {
        if i == 1 {
          assert_eq!(1, child.get_data().statistics.visits());
          assert_eq!(
            0,
            child
              .get_data()
              .statistics
              .score(crate::statistics::two_player::Player::One)
          );
          assert_eq!(
            1,
            child
              .get_data()
              .statistics
              .score(crate::statistics::two_player::Player::Two)
          );
        } else {
          assert_eq!(0, child.get_data().statistics.visits());
          assert_eq!(
            0,
            child
              .get_data()
              .statistics
              .score(crate::statistics::two_player::Player::One)
          );
          assert_eq!(
            0,
            child
              .get_data()
              .statistics
              .score(crate::statistics::two_player::Player::Two)
          );
        }
      }
    }

    search_graph::view::of_graph(&mut graph, |view| {
      RolloutPhase::initialize(
        default_rng(),
        default_settings(),
        default_game_state(),
        view,
      )
      .rollout::<ucb::Rollout>()
      .unwrap()
      .score::<simulation::RandomSimulator>()
      .unwrap()
      .backprop::<backprop::FirstParentSelector>()
      .expand();
    });
    {
      let node = graph.find_node(&Default::default()).unwrap();
      // for (i, child) in node.get_child_list().iter().enumerate() {
      //   if i == 1 {
      //     assert_eq!(1, child.get_data().statistics.visits());
      //     assert_eq!(
      //       0,
      //       child
      //         .get_data()
      //         .statistics
      //         .score(crate::statistics::two_player::Player::One)
      //     );
      //     assert_eq!(
      //       1,
      //       child
      //         .get_data()
      //         .statistics
      //         .score(crate::statistics::two_player::Player::Two)
      //     );
      //   } else {
      //     assert_eq!(0, child.get_data().statistics.visits());
      //     assert_eq!(
      //       0,
      //       child
      //         .get_data()
      //         .statistics
      //         .score(crate::statistics::two_player::Player::One)
      //     );
      //     assert_eq!(
      //       0,
      //       child
      //         .get_data()
      //         .statistics
      //         .score(crate::statistics::two_player::Player::Two)
      //     );
      //   }
      // }
    }

    search_graph::view::of_graph(&mut graph, |view| {
      let mut rollout = RolloutPhase::initialize(
        default_rng(),
        default_settings(),
        default_game_state(),
        view,
      );
      for _ in 0..10000 {
        let score = rollout.rollout::<ucb::Rollout>().unwrap();
        let backprop = score.score::<simulation::RandomSimulator>().unwrap();
        let expand = backprop.backprop::<backprop::FirstParentSelector>();
        rollout = expand.expand();
      }
    });

    for child in graph.find_node(&default_game_state()).unwrap().get_child_list().iter() {
      let statistics = &child.get_data().statistics;
      // assert!(statistics.visits() > 500);
      assert_eq!(statistics.score(crate::statistics::two_player::Player::One), 250);
      assert_eq!(statistics.score(crate::statistics::two_player::Player::Two), 250);
    }
  }
}
