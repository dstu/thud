use crate::actions::Action;
use crate::Role;
use mcts::{statistics, SearchSettings};
use rand::Rng;
use search_graph;
use std::{cmp, mem};

#[derive(Clone, Debug)]
pub struct Game {}

impl statistics::two_player::PlayerMapping for Role {
  fn player_one() -> Self {
    Role::Dwarf
  }
  fn player_two() -> Self {
    Role::Troll
  }
  fn resolve_player(&self) -> statistics::two_player::Player {
    match *self {
      Role::Dwarf => statistics::two_player::Player::One,
      Role::Troll => statistics::two_player::Player::Two,
    }
  }
}

impl mcts::game::State for crate::state::State {
  type Action = Action;
  type PlayerId = Role;

  fn active_player(&self) -> &Role {
    &self.active_role()
  }

  fn for_actions<F>(&self, mut f: F)
  where
    F: FnMut(Action) -> mcts::game::LoopControl,
  {
    let mut actions = self.actions();
    loop {
      match actions.next() {
        Some(a) => {
          match f(a) {
            mcts::game::LoopControl::Continue => continue,
            mcts::game::LoopControl::Break => break,
          }
        }
        None => break,
      }
    }
  }

  fn do_action(&mut self, action: &Action) {
    self.do_action(action);
  }
}

impl mcts::game::Game for Game {
  type Action = Action;
  type PlayerId = Role;
  type Payoff = statistics::two_player::ScoredPayoff;
  type State = crate::state::State;
  type Statistics = statistics::two_player::ScoredStatistics<Role>;

  fn payoff_of(state: &Self::State) -> Option<Self::Payoff> {
    if state.terminated() {
      let mut dwarf = 0u32;
      let mut troll = 0u32;
      for (_, c) in state.cells().cells_iter() {
        match c {
          crate::board::Content::Occupied(crate::board::Token::Dwarf) => dwarf += 1,
          crate::board::Content::Occupied(crate::board::Token::Troll) => troll += 4,
          _ => (),
        }
      }
      Some(statistics::two_player::ScoredPayoff {
        visits: 1,
        score_one: dwarf,
        score_two: troll,
      })
    } else {
      None
    }
  }
}

/// Controls how a game action is selected by the [MCTS
/// agent](struct.Agent.html) after MCTS search has terminated and all
/// statistics have been gathered.
#[derive(Debug, Clone, Copy)]
pub enum ActionSelect {
  /// Select the action that was visited the most times.
  VisitCount,
  /// Select the action with the best UCB score.
  Ucb,
}

/// Controls how graph compaction is done by the [MCTS agent](struct.Agent.html)
/// before each round of MCTS search.
#[derive(Debug, Clone, Copy)]
pub enum GraphCompact {
  /// Prune the search graph so that the current game state and all its
  /// descendants are retained, but game states that are not reachable from the
  /// current game state are removed.
  Prune,
  /// Clear the entire search graph.
  Clear,
  /// Retain the entire contents of the search graph.
  Retain,
}

type SearchGraph =
  search_graph::Graph<crate::state::State, mcts::graph::VertexData, mcts::graph::EdgeData<Game>>;

pub struct Agent<R: Rng> {
  settings: SearchSettings,
  iterations: u32,
  rng: R,
  action_select: ActionSelect,
  graph_compact: GraphCompact,
  graph: SearchGraph,
}

impl<R: Rng> Agent<R> {
  pub fn new(
    settings: SearchSettings,
    iterations: u32,
    rng: R,
    action_select: ActionSelect,
    graph_compact: GraphCompact,
  ) -> Self {
    Agent {
      settings,
      iterations,
      rng,
      action_select,
      graph_compact,
      graph: SearchGraph::new(),
    }
  }
}

impl<R: Rng + Send> crate::agent::Agent for Agent<R> {
  fn propose_action(&mut self, state: &crate::state::State) -> crate::agent::Result {
    match self.graph_compact {
      GraphCompact::Prune => {
        if let Some(node) = self.graph.find_node_mut(state) {
          search_graph::view::of_node(node, |view, node| {
            view.retain_reachable_from(Some(node).into_iter());
          });
        } else {
          mem::swap(&mut self.graph, &mut SearchGraph::new());
        }
      }
      GraphCompact::Clear => mem::swap(&mut self.graph, &mut SearchGraph::new()),
      GraphCompact::Retain => (),
    }

    // Borrow/copy stuff out of self because the closure passed to of_graph
    // can't borrow self.
    let (rng, graph, settings, iterations, action_select) = (
      &mut self.rng,
      &mut self.graph,
      self.settings.clone(),
      self.iterations,
      self.action_select,
    );
    search_graph::view::of_graph(graph, |view| -> crate::agent::Result {
      let mut rollout = mcts::RolloutPhase::initialize(rng, settings, state.clone(), view);
      for _ in 0..iterations {
        let scoring = match rollout.rollout::<mcts::ucb::Rollout>() {
          Ok(s) => s,
          Err(e) => return Err(Box::new(e)),
        };
        let backprop = match scoring.score::<mcts::simulation::RandomSimulator>() {
          Ok(b) => b,
          Err(e) => return Err(Box::new(e)),
        };
        rollout = backprop
          .backprop::<mcts::ucb::BestParentBackprop>()
          .expand();
      }
      let (rng, view) = rollout.recover_components();

      let root = view.find_node(state).unwrap();
      let best_action: Action = match action_select {
        ActionSelect::Ucb => {
          match mcts::ucb::find_best_child(&view, root, settings.explore_bias, rng) {
            Ok(child) => view[child].action().clone(),
            Err(e) => return Err(Box::new(e)),
          }
        }
        ActionSelect::VisitCount => {
          let mut children = view.children(root);
          let mut best_child = children.next().unwrap();
          let mut best_child_visits = view[best_child].statistics.visits();
          let mut reservoir_count = 1u32;
          for child in children {
            let visits = view[child].statistics.visits();
            match visits.cmp(&best_child_visits) {
              cmp::Ordering::Less => continue,
              cmp::Ordering::Equal => {
                reservoir_count += 1;
                if !rng.gen_bool(1.0f64 / (reservoir_count as f64)) {
                  continue;
                }
              }
              cmp::Ordering::Greater => reservoir_count = 1,
            }
            best_child = child;
            best_child_visits = visits;
          }
          view[best_child].action().clone()
        }
      };
      Ok(best_action)
    })
  }
}
