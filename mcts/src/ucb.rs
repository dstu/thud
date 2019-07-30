//! Upper confidence bound (UCB1) algorithm for graph search.

use crate::backprop::BackpropSelector;
use crate::game::{Game, State, Statistics};
use crate::graph::{EdgeData, VertexData};
use crate::rollout::RolloutSelector;
use log::{error, trace};
use rand::Rng;
use search_graph;

use std::cmp::Ordering;
use std::error::Error;
use std::f64;
use std::fmt;
use std::result::Result;

/// Represents success when computing the UCB score for a child.
pub enum UcbSuccess<'id> {
  /// No (finite) value can be computed, but the UCB policy indicates that
  /// this child should be selected. E.g., the child has not yet been visited.
  Select(search_graph::view::EdgeRef<'id>),
  /// A value is computed.
  Value(search_graph::view::EdgeRef<'id>, f64),
}

/// Represents error conditions when computing the UCB score for a child.
#[derive(Clone, Debug)]
pub enum UcbError {
  /// An error was encountered during computation of UCB score (such as
  /// encountering a `None` result when `PartialCmp`'ing two floating-point
  /// values).
  InvalidComputation,
}

/// Lazy iterator over UCB scores for a series of edges.
pub struct EdgeUcbIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  log_parent_visits: f64,
  explore_bias: f64,
  graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  edges: I,
}

impl fmt::Display for UcbError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      UcbError::InvalidComputation => write!(f, "Numerical error when computing UCB score"),
    }
  }
}

impl Error for UcbError {
  fn description(&self) -> &str {
    match *self {
      UcbError::InvalidComputation => "invalid computation",
    }
  }
}

impl<'a, 'b, 'id, G, I> EdgeUcbIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  /// Constructs a new `EdgeUcbIter` that will compute UCB scores using the
  /// given constants. All floating-point values are assumed to be valid
  /// floating-point values and positive.
  ///
  ///  - `log_parent_visits`: ln(visits to parent vertex).
  ///  - `explore_bias`: scalar bias controlling tradeoff between search width
  ///    and search depth (lower = wider, higher = deeper).
  ///  - `role`: the game role whose score to maximize.
  ///  - `edges`: an iterator over edge handles for which to compute UCB
  ///    scores. This should usually be a list of child edges which share a
  ///    parent vertex.
  pub fn new(
    log_parent_visits: f64,
    explore_bias: f64,
    graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
    edges: I,
  ) -> Self {
    EdgeUcbIter {
      log_parent_visits: log_parent_visits,
      explore_bias: explore_bias,
      graph,
      edges: edges,
    }
  }
}

impl<'a, 'b, 'id, G, I> Iterator for EdgeUcbIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  type Item = Result<UcbSuccess<'id>, UcbError>;

  fn next(&mut self) -> Option<Result<UcbSuccess<'id>, UcbError>> {
    self.edges.next().map(|e| {
      if self.graph.edge_data(e).statistics.visits() == 0 {
        Ok(UcbSuccess::Select(e))
      } else {
        Ok(child_score(
          self.log_parent_visits,
          self.explore_bias,
          self.graph,
          e,
        ))
      }
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.edges.size_hint()
  }
}

/// Returns the UCB policy result for the given values.
pub fn child_score<'a, 'id, G: Game>(
  log_parent_visits: f64,
  explore_bias: f64,
  graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  child: search_graph::view::EdgeRef<'id>,
) -> UcbSuccess<'id> {
  let statistics = &graph.edge_data(child).statistics;
  if statistics.visits() == 0 {
    UcbSuccess::Select(child)
  } else {
    let child_visits = statistics.visits() as f64;
    let child_score =
      statistics.score(graph.node_state(graph.edge_source(child)).active_player()) as f64;
    let ucb =
      child_score / child_visits + explore_bias * f64::sqrt(log_parent_visits / child_visits);
    UcbSuccess::Value(child, ucb)
  }
}

/// Returns `true` iff `e` could be selected by the UCB policy during rollout
/// from its parent vertex. Assumes we haven't yet altered the parent vertex
/// statistics. (Callers should ensure that this is not called more than once
/// for a given parent vertex.)
///
/// "Could be" reflects how this is different from calling
/// `find_best_child_edge`: it is possible (and common) for multiple child
/// edges of a vertex to be best children (as when there are multiple children
/// that have not yet been explored). This is dealt with in
/// `find_best_child_edge` by randomly choosing one of the best
/// children. But when doing backpropagation on a full game state graph (not
/// just a tree), we want to know all of the parent edges which could have
/// rolled out to a given child.
pub fn is_best_child<'a, 'id, G: Game>(
  graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  e: search_graph::view::EdgeRef<'id>,
  explore_bias: f64,
) -> bool {
  let statistics = &graph.edge_data(e).statistics;
  // trace!("is_best_child: edge {} has {} visits", e.get_id(), stats.visits);
  if statistics.visits() == 0 {
    // Edge has been visited, but statistics aren't yet updated.
    // trace!("is_best_child: edge {} is a best child because stats.visits == 0", e.get_id());
    return true;
  }
  let parent = graph.edge_source(e);
  // trace!("is_best_child: edge {} (from node {}) has {} siblings", e.get_id(), parent.get_id(), siblings.len());
  let log_parent_visits = {
    let mut parent_visits = 0;
    for child_edge in graph.children(parent) {
      parent_visits += graph.edge_data(child_edge).statistics.visits();
    }
    f64::ln(parent_visits as f64)
  };
  let mut edge_ucb = None;
  let mut best_ucb = ::std::f64::MIN;
  let ucb_iter = EdgeUcbIter::new(
    log_parent_visits,
    explore_bias,
    graph,
    graph.children(parent),
  );
  // Scan through siblings to find the maximum UCB score. This is
  // short-circuited using a lazy iterator to ameliorate the O(n) running
  // time.
  for ucb in ucb_iter {
    match ucb {
      Ok(UcbSuccess::Select(sibling)) if e == sibling => {
        // trace!("is_best_child: edge {} is best child by fiat of score computation", e.get_id());
        return true;
      }
      Ok(UcbSuccess::Select(_)) => {
        if let Some(_) = edge_ucb {
          // trace!("is_best_child: edge {} has a ucb of {}, but sibling edge {} is best child by fiat of score computation", e.get_id(), u, sibling.get_id());
          return false;
        }
      }
      Ok(UcbSuccess::Value(sibling, score)) => {
        if sibling == e {
          if best_ucb > score {
            // trace!("is_best_child: found ucb {} for edge {}, but a sibling has a higher ucb of {}", score, e.get_id(), best_ucb);
            return false;
          }
          edge_ucb = Some(score)
        } else if let Some(u) = edge_ucb {
          if score > u {
            // trace!("is_best_child: found ucb {} for edge {}, but a sibling has a higher ucb of {}", score, e.get_id(), best_ucb);
            return false;
          }
        }
        if score > best_ucb {
          best_ucb = score;
        }
      }
      Err(e) => panic!("error {:?} computing ucb for best child search", e),
    }
  }
  match edge_ucb {
    Some(u) if u >= best_ucb => {
      // Target edge has a UCB score which matches the maximum we found.
      // trace!("is_best_child: edge {} has a ucb of {}, which matches the max sibling ucb of {}", e.get_id(), u, best_ucb);
      true
    }
    _ => {
      // ThudEdges are not best children by default.
      // trace!("is_best_child: edge {} has a ucb of {:?}, which does not match the max sibling ucb of {}", e.get_id(), edge_ucb, best_ucb);
      false
    }
  }
}

/// Returns the child edge of `parent` that is best according to the UCB1
/// criterion.
///
/// This function will panic if `parent` has no children.
pub fn find_best_child<'a, 'id, G, R>(
  graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  parent: search_graph::view::NodeRef<'id>,
  explore_bias: f64,
  rng: &mut R,
) -> Result<search_graph::view::EdgeRef<'id>, UcbError>
where
  G: Game,
  R: Rng,
{
  let log_parent_visits = {
    let mut parent_visits = 0;
    for child in graph.children(parent) {
      parent_visits += graph.edge_data(child).statistics.visits();
    }
    if parent_visits == 0 {
      // When we visit a vertex for the first time, it will have zero visits.
      0.0
    } else {
      // Otherwise, it should be positive.
      f64::ln(parent_visits as f64)
    }
  };
  let mut sampling_count = 0u32;
  let mut ucb_iter = EdgeUcbIter::new(
    log_parent_visits,
    explore_bias,
    graph,
    graph.children(parent),
  );
  let mut best: search_graph::view::EdgeRef<'id>;
  let mut best_ucb: f64;
  match ucb_iter.next().expect("vertex has no children")? {
    UcbSuccess::Select(e) => {
      best = e;
      best_ucb = f64::MAX;
    }
    UcbSuccess::Value(e, v) => {
      best = e;
      best_ucb = v;
    }
  }

  for ucb in ucb_iter {
    let (edge, value) = match ucb? {
      UcbSuccess::Select(e) => (e, f64::MAX),
      UcbSuccess::Value(e, v) => (e, v),
    };
    match value.partial_cmp(&best_ucb) {
      None => {
        error!("find_best_child: invalid floating-point comparison");
        return Err(UcbError::InvalidComputation);
      }
      Some(Ordering::Greater) => {
        trace!("find_best_child_edge: new best action with score {}", value);
        best = edge;
        best_ucb = value;
        sampling_count = 1;
      }
      Some(Ordering::Equal) => {
        // We use reservoir sampling to break ties.
        trace!(
          "find_best_child: found action with identical score {}; sampling to break tie",
          value
        );
        sampling_count += 1;
        if rng.gen_ratio(1, sampling_count) {
          trace!("find_best_child: broke tie by selecting new action");
          best = edge;
        } else {
          trace!("find_best_child: broke tie by keeping old action");
        }
      }
      Some(Ordering::Less) => (),
    }
  }
  Ok(best)
}

/// [Rollout selector](../rollout/trait.RolloutSelector.html) that chooses a
/// child with the highest the UCB1 score.
///
/// If more than one child has the same rollout score, chooses one such child at
/// random.
pub struct Rollout {
  explore_bias: f64,
}

impl<'a> From<&'a crate::SearchSettings> for Rollout {
  fn from(settings: &'a crate::SearchSettings) -> Self {
    Rollout {
      explore_bias: settings.explore_bias,
    }
  }
}

impl RolloutSelector for Rollout {
  type Error = UcbError;

  fn select<'a, 'id, G: Game, R: Rng>(
    &self,
    graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
    parent: search_graph::view::NodeRef<'id>,
    rng: &mut R,
  ) -> Result<search_graph::view::EdgeRef<'id>, UcbError> {
    find_best_child(graph, parent, self.explore_bias, rng)
  }
}

/// [Backprop selector](../backprop/trait.BackpropSelector.html) that traverses
/// upward edges that would have been selected by the [UCB1
/// rollout](struct.Rollout.html) policy.
pub struct BestParentBackprop {
  explore_bias: f64,
}

impl<'a> From<&'a crate::SearchSettings> for BestParentBackprop {
  fn from(settings: &'a crate::SearchSettings) -> Self {
    BestParentBackprop {
      explore_bias: settings.explore_bias,
    }
  }
}

impl<'id> BackpropSelector<'id> for BestParentBackprop {
  // TODO: this requires an allocation. We have everything in place for this
  // selector to be allocation-free, except Rust doesn't yet support
  // ATCs/HKTs. We need ATC support because this iterator type will have its
  // lifetime constrained by the borrow of `graph` in the select method, but
  // that lifetime isn't known statically.
  type Items = std::vec::IntoIter<search_graph::view::EdgeRef<'id>>;

  fn select<G: Game, R: Rng>(
    &self,
    graph: &search_graph::view::View<'_, 'id, G::State, VertexData, EdgeData<G>>,
    node: search_graph::view::NodeRef<'id>,
    _payoff: &G::Payoff,
    _rng: &mut R,
  ) -> Self::Items {
    let result: Vec<search_graph::view::EdgeRef<'id>> = graph
      .parents(node)
      .filter(|&parent_edge| is_best_child(graph, parent_edge, self.explore_bias))
      .collect();
    result.into_iter()
  }
}
