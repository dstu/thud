//! Single-threaded Monte Carlo tree search on directed acyclic graphs.

pub mod backprop;
pub mod game;
pub mod graph;
pub mod rollout;
pub mod simulation;
pub mod statistics;
pub mod ucb;

#[cfg(test)]
pub(crate) mod tictactoe;

use crate::backprop::BackpropSelector;
use crate::game::{Game, State};
use crate::graph::{EdgeData, VertexData};
use crate::rollout::RolloutSelector;
use crate::simulation::Simulator;

use std::convert::From;
use std::result::Result;

use log::trace;
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
    trace!("initializing rollout phase to state: {:?}", root_state);
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
  ) -> Result<ScoringPhase<'a, 'id, R, G>, rollout::RolloutError<G, S::Error>> {
    let result = rollout::rollout(
      &self.graph,
      self.root_node,
      S::from(&self.settings),
      &mut self.rng,
    );
    trace!("rollout finds result {:?}", result);
    result.map(|node| {
      trace!("rollout result has target node: {:?}", node);
      trace!(
        "rollout result has state: {:?}",
        self.graph.node_state(node)
      );
      ScoringPhase {
        rng: self.rng,
        settings: self.settings,
        graph: self.graph,
        root_node: self.root_node,
        rollout_node: node,
      }
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
    trace!("scoring phase finds payoff {:?}", payoff);
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
    if self.graph.node_data(self.rollout_node).mark_expanded() {
      trace!("rollout node was already marked as expanded; ExpandPhase does nothing");
    } else {
      self
        .graph
        .node_state(self.rollout_node)
        .clone()
        .for_actions(|action| {
          trace!("ExpandPhase adds edge for action {:?}", action);
          let mut child_state = self.graph.node_state(self.rollout_node).clone();
          trace!("ExpandState old state: {:?}", child_state);
          child_state.do_action(&action);
          trace!("ExpandState new state: {:?}", child_state);
          let child = match self.graph.find_node(&child_state) {
            Some(n) => {
              trace!("ExpandState expanded to existing game state");
              n
            }
            None => {
              trace!("ExpandState expanded to new game state");
              self
                .graph
                .append_node(child_state.clone(), Default::default())
            }
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
      // Search graph should consist of root, edges for each possible move, and
      // leaves for the result of each move.
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
      // Two levels of should be expanded.
      assert_eq!(18, graph.vertex_count());
      assert_eq!(17, graph.edge_count());
      let node = graph.find_node(&Default::default()).unwrap();
      assert!(node.is_root());
      assert!(!node.is_leaf());
      assert_eq!(9, node.get_child_list().len());
      for (i, child) in node.get_child_list().iter().enumerate() {
        if i == 7 {
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
  }

  #[test]
  fn integration_test_first_parent_selector() {
    let mut graph = Graph::new();
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

    assert_eq!(3295, graph.vertex_count());
    assert_eq!(5361, graph.edge_count());

    let child_statistics: Vec<&crate::statistics::two_player::ScoredStatistics<_>> =
    graph
      .find_node(&default_game_state())
      .unwrap()
      .get_child_list()
      .iter()
      .map(|c| &c.get_data().statistics)
      .collect();
    assert_eq!(108, child_statistics[0].visits());
    assert_eq!(28, child_statistics[0].score(crate::statistics::two_player::Player::One));
    assert_eq!(78, child_statistics[0].score(crate::statistics::two_player::Player::Two));
    assert_eq!(209, child_statistics[1].visits());
    assert_eq!(153, child_statistics[1].score(crate::statistics::two_player::Player::One));
    assert_eq!(50, child_statistics[1].score(crate::statistics::two_player::Player::Two));
    assert_eq!(754, child_statistics[2].visits());
    assert_eq!(557, child_statistics[2].score(crate::statistics::two_player::Player::One));
    assert_eq!(178, child_statistics[2].score(crate::statistics::two_player::Player::Two));
    assert_eq!(2023, child_statistics[3].visits());
    assert_eq!(1548, child_statistics[3].score(crate::statistics::two_player::Player::One));
    assert_eq!(459, child_statistics[3].score(crate::statistics::two_player::Player::Two));
    assert_eq!(2105, child_statistics[4].visits());
    assert_eq!(834, child_statistics[4].score(crate::statistics::two_player::Player::One));
    assert_eq!(1181, child_statistics[4].score(crate::statistics::two_player::Player::Two));
    assert_eq!(1172, child_statistics[5].visits());
    assert_eq!(574, child_statistics[5].score(crate::statistics::two_player::Player::One));
    assert_eq!(590, child_statistics[5].score(crate::statistics::two_player::Player::Two));
    assert_eq!(953, child_statistics[6].visits());
    assert_eq!(483, child_statistics[6].score(crate::statistics::two_player::Player::One));
    assert_eq!(282, child_statistics[6].score(crate::statistics::two_player::Player::Two));
    assert_eq!(806, child_statistics[7].visits());
    assert_eq!(382, child_statistics[7].score(crate::statistics::two_player::Player::One));
    assert_eq!(410, child_statistics[7].score(crate::statistics::two_player::Player::Two));
    assert_eq!(1869, child_statistics[8].visits());
    assert_eq!(678, child_statistics[8].score(crate::statistics::two_player::Player::One));
    assert_eq!(913, child_statistics[8].score(crate::statistics::two_player::Player::Two));
  }

  #[test]
  fn integration_test_random_parent_selector() {
    let mut graph = Graph::new();
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
        let expand = backprop.backprop::<backprop::RandomParentSelector>();
        rollout = expand.expand();
      }
    });

    assert_eq!(5795, graph.vertex_count());
    assert_eq!(13874, graph.edge_count());

    let child_statistics: Vec<&crate::statistics::two_player::ScoredStatistics<_>> =
    graph
      .find_node(&default_game_state())
      .unwrap()
      .get_child_list()
      .iter()
      .map(|c| &c.get_data().statistics)
      .collect();
    assert_eq!(1204, child_statistics[0].visits());
    assert_eq!(708, child_statistics[0].score(crate::statistics::two_player::Player::One));
    assert_eq!(347, child_statistics[0].score(crate::statistics::two_player::Player::Two));
    assert_eq!(790, child_statistics[1].visits());
    assert_eq!(396, child_statistics[1].score(crate::statistics::two_player::Player::One));
    assert_eq!(266, child_statistics[1].score(crate::statistics::two_player::Player::Two));
    assert_eq!(983, child_statistics[2].visits());
    assert_eq!(553, child_statistics[2].score(crate::statistics::two_player::Player::One));
    assert_eq!(249, child_statistics[2].score(crate::statistics::two_player::Player::Two));
    assert_eq!(829, child_statistics[3].visits());
    assert_eq!(471, child_statistics[3].score(crate::statistics::two_player::Player::One));
    assert_eq!(258, child_statistics[3].score(crate::statistics::two_player::Player::Two));
    assert_eq!(2015, child_statistics[4].visits());
    assert_eq!(1224, child_statistics[4].score(crate::statistics::two_player::Player::One));
    assert_eq!(388, child_statistics[4].score(crate::statistics::two_player::Player::Two));
    assert_eq!(795, child_statistics[5].visits());
    assert_eq!(449, child_statistics[5].score(crate::statistics::two_player::Player::One));
    assert_eq!(242, child_statistics[5].score(crate::statistics::two_player::Player::Two));
    assert_eq!(1035, child_statistics[6].visits());
    assert_eq!(578, child_statistics[6].score(crate::statistics::two_player::Player::One));
    assert_eq!(287, child_statistics[6].score(crate::statistics::two_player::Player::Two));
    assert_eq!(1002, child_statistics[7].visits());
    assert_eq!(581, child_statistics[7].score(crate::statistics::two_player::Player::One));
    assert_eq!(271, child_statistics[7].score(crate::statistics::two_player::Player::Two));
    assert_eq!(1346, child_statistics[8].visits());
    assert_eq!(798, child_statistics[8].score(crate::statistics::two_player::Player::One));
    assert_eq!(399, child_statistics[8].score(crate::statistics::two_player::Player::Two));
  }

  #[test]
  fn integration_test_best_parent_selector() {
    let mut graph = Graph::new();
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
        let expand = backprop.backprop::<ucb::BestParentBackprop>();
        rollout = expand.expand();
      }
    });

    assert_eq!(4471, graph.vertex_count());
    assert_eq!(10889, graph.edge_count());

    let child_statistics: Vec<&crate::statistics::two_player::ScoredStatistics<_>> =
    graph
      .find_node(&default_game_state())
      .unwrap()
      .get_child_list()
      .iter()
      .map(|c| &c.get_data().statistics)
      .collect();
    assert_eq!(229, child_statistics[0].visits());
    assert_eq!(138, child_statistics[0].score(crate::statistics::two_player::Player::One));
    assert_eq!(53, child_statistics[0].score(crate::statistics::two_player::Player::Two));
    assert_eq!(73, child_statistics[1].visits());
    assert_eq!(33, child_statistics[1].score(crate::statistics::two_player::Player::One));
    assert_eq!(30, child_statistics[1].score(crate::statistics::two_player::Player::Two));
    assert_eq!(322, child_statistics[2].visits());
    assert_eq!(203, child_statistics[2].score(crate::statistics::two_player::Player::One));
    assert_eq!(83, child_statistics[2].score(crate::statistics::two_player::Player::Two));
    assert_eq!(90, child_statistics[3].visits());
    assert_eq!(44, child_statistics[3].score(crate::statistics::two_player::Player::One));
    assert_eq!(32, child_statistics[3].score(crate::statistics::two_player::Player::Two));
    assert_eq!(9033, child_statistics[4].visits());
    assert_eq!(7693, child_statistics[4].score(crate::statistics::two_player::Player::One));
    assert_eq!(945, child_statistics[4].score(crate::statistics::two_player::Player::Two));
    assert_eq!(74, child_statistics[5].visits());
    assert_eq!(34, child_statistics[5].score(crate::statistics::two_player::Player::One));
    assert_eq!(35, child_statistics[5].score(crate::statistics::two_player::Player::Two));
    assert_eq!(118, child_statistics[6].visits());
    assert_eq!(62, child_statistics[6].score(crate::statistics::two_player::Player::One));
    assert_eq!(43, child_statistics[6].score(crate::statistics::two_player::Player::Two));
    assert_eq!(98, child_statistics[7].visits());
    assert_eq!(49, child_statistics[7].score(crate::statistics::two_player::Player::One));
    assert_eq!(36, child_statistics[7].score(crate::statistics::two_player::Player::Two));
    assert_eq!(180, child_statistics[8].visits());
    assert_eq!(104, child_statistics[8].score(crate::statistics::two_player::Player::One));
    assert_eq!(41, child_statistics[8].score(crate::statistics::two_player::Player::Two));
  }
}
