//! Upper confidence bound (UCB1) algorithm for graph search.

use super::{EdgeData, Game, Payoff, State, Statistics, VertexData};
use log::error;
use rand::Rng;
use search_graph;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::result::Result;

/// Represents success when computing the UCB score for a child.
pub enum UcbSuccess<'a> {
  /// No (finite) value can be computed, but the UCB policy indicates that
  /// this child should be selected. E.g., the child has not yet been visited.
  Select(search_graph::view::EdgeRef<'a>),
  /// A value is computed.
  Value(
    search_graph::view::EdgeRef<'a>,
    f64,
  ),
}

/// Represents error conditions when computing the UCB score for a child.
#[derive(Clone, Debug)]
pub enum UcbError {
  /// There are no children to select from.
  NoChildren,
  /// An error was encountered during computation of UCB score (such as
  /// encountering a `None` result when `PartialCmp`'ing two floating-point
  /// values).
  InvalidComputation,
}

/// Lazy iterator over UCB scores for a series of edges.
pub struct EdgeUcbIter<'a, 'b, G, I>
where
  'a: 'b,
  G: 'a + Game,
  I: 'b + Iterator<Item = search_graph::view::EdgeRef<'a>>,
{
  log_parent_visits: f64,
  explore_bias: f64,
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  edges: I,
}

impl fmt::Display for UcbError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      UcbError::NoChildren => write!(f, "Vertex has no children"),
      UcbError::InvalidComputation => write!(f, "Numerical error when computing UCB score"),
    }
  }
}

impl Error for UcbError {
  fn description(&self) -> &str {
    match *self {
      UcbError::NoChildren => "no children",
      UcbError::InvalidComputation => "invalid computation",
    }
  }
}

impl<'a, 'b, G, I> EdgeUcbIter<'a, 'b, G, I>
where
  'a: 'b,
  G: Game,
  I: 'b + Iterator<Item = search_graph::view::EdgeRef<'a>>,
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
  pub fn new(log_parent_visits: f64, explore_bias: f64, graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>, edges: I) -> Self {
    EdgeUcbIter {
      log_parent_visits: log_parent_visits,
      explore_bias: explore_bias,
      graph,
      edges: edges,
    }
  }
}

impl<'a, 'b, G, I> Iterator for EdgeUcbIter<'a, 'b, G, I>
where
  'a: 'b,
  G: Game,
  I: 'b + Iterator<Item = search_graph::view::EdgeRef<'a>>,
{
  type Item = Result<UcbSuccess<'a>, UcbError>;

  fn next(&mut self) -> Option<Result<UcbSuccess<'a>, UcbError>> {
    self.edges.next().map(|e| {
      let payoff = self.graph.edge_data(e).statistics.as_payoff();
      if payoff.visits() == 0 {
        Ok(UcbSuccess::Select(e))
      } else {
        Ok(child_score(self.log_parent_visits, self.explore_bias, self.graph, e))
      }
    })
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.edges.size_hint()
  }
}

/// Returns the UCB policy result for the given values.
pub fn child_score<'a, G: Game>(
  log_parent_visits: f64,
  explore_bias: f64,
  graph: &search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  child: search_graph::view::EdgeRef<'a>,
) -> UcbSuccess<'a> {
  let payoff = graph.edge_data(child).statistics.as_payoff();
  if payoff.visits() == 0 {
    UcbSuccess::Select(child)
  } else {
    let child_visits = payoff.visits() as f64;
    let child_payoff = payoff.score(graph.node_state(graph.edge_source(child)).active_player()) as f64;
    let ucb =
      child_payoff / child_visits + explore_bias * f64::sqrt(log_parent_visits / child_visits);
    UcbSuccess::Value(child, ucb)
  }
}

/// Returns `true` iff `e` could be selected by the UCB policy during rollout
/// from its parent vertex. Assumes we haven't yet altered the parent vertex
/// statistics. (Callers should ensure that this is not called more than once
/// for a given parent vertex.)
///
/// "Could be" reflects how this is different from calling
/// `find_best_child_edge_index`: it is possible (and common) for multiple child
/// edges of a vertex to be best children (as when there are multiple children
/// that have not yet been explored). This is dealt with in
/// `find_best_child_edge_index` by randomly choosing one of the best
/// children. But when doing backpropagation on a full game state graph (not
/// just a tree), we want to know all of the parent edges which could have
/// rolled out to a given child.
pub fn is_best_child<'a, 'b, G: 'a + Game>(
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  e: search_graph::view::EdgeRef<'a>,
  explore_bias: f64,
) -> bool where 'a: 'b {
  let payoff = graph.edge_data(e).statistics.as_payoff();
  // trace!("is_best_child: edge {} has {} visits", e.get_id(), stats.visits);
  if payoff.visits() == 0 {
    // Edge has been visited, but statistics aren't yet updated.
    // trace!("is_best_child: edge {} is a best child because stats.visits == 0", e.get_id());
    return true;
  }
  let parent = graph.edge_source(e);
  // trace!("is_best_child: edge {} (from node {}) has {} siblings", e.get_id(), parent.get_id(), siblings.len());
  let log_parent_visits = {
    let mut parent_visits = 0;
    for child_edge in graph.children(parent) {
      parent_visits += graph.edge_data(child_edge).statistics.as_payoff().visits();
    }
    f64::ln(parent_visits as f64)
  };
  let mut edge_ucb = None;
  let mut best_ucb = ::std::f64::MIN;
  let ucb_iter = EdgeUcbIter::new(log_parent_visits, explore_bias, graph, graph.children(parent));
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
  //     match sibling_edge.get_taras_payoff() {
  //         search_graph::Target::Unexpanded(_) => {
  //             // This sibling has not yet been visited. We know that this edge
  //             // has been visited, and we will always visit all siblings at
  //             // least once before returning to this one.
  //             trace!("is_best_child: edge {} is not a best child because it has an unexpanded sibling and its own visit count is {}",
  //                    e.get_id(), stats.visits);
  //             return false
  //         },
  //         search_graph::Target::Expanded(_) => {
  //             let sibling_stats = sibling_edge.get_data().statistics.as_payoff();
  //             match score(log_parent_visits, sibling_stats.visits as f64,
  //                         stats.payoff.score(player) as f64, explore_bias) {
  //                 Ok(UcbSuccess::Select) => {
  //                     // Score computation short-circuits UCB to select this
  //                     // sibling, so we select it iff it is the edge we're
  //                     // considering. This only happens when the sibling
  //                     // hasn't been visited yet, and we checked for that
  //                     // above, but we handle the case again here to complete
  //                     // the match.
  //                     trace!("is_best_child: edge {} best child determined by weird edge case", e.get_id());
  //                     return sibling_edge.get_id() == e.get_id()
  //                 },
  //                 Ok(UcbSuccess::Value(ucb)) => {
  //                     trace!("is_best_child({:?}): edge {} (parent node {}) has ucb {}",
  //                            player, sibling_edge.get_id(), sibling_edge.get_source().get_id(), ucb);
  //                     if sibling_edge.get_id() == e.get_id() {
  //                         if best_ucb > ucb {
  //                             // We have already seen an edge with a greater
  //                             // UCB score.
  //                             trace!("edge {} is not a best child because we have seen a greater UCB score",
  //                                    e.get_id());
  //                             return false
  //                         }
  //                         edge_ucb = Some(ucb);
  //                     }
  //                     if let Some(u) = edge_ucb {
  //                         if ucb > u {
  //                             // This edge has a greater UCB score.
  //                             trace!("is_best_child({:?}): ucb of {} exceeds {} of edge",
  //                                    player, ucb, u);
  //                             return false
  //                         }
  //                     }
  //                     if ucb > best_ucb {
  //                         best_ucb = ucb;
  //                     }
  //                 },
  //                 Err(e) => panic!("error {:?} computing ucb for best child search", e),
  //             }
  //         },
  //     }
  // }
  // trace!("is_best_child: edge ucb of {:?} vs. best_ucb of {}", edge_ucb, best_ucb);
  // match edge_ucb {
  //     Some(u) if u >= best_ucb =>
  //         // Target edge has a UCB score which matches the maximum we found.
  //         true,
  //     _ =>
  //         // ThudEdges are not best children by default.
  //         false,
  // }
}

pub fn find_best_child_edge_index<'a, 'b, G, R>(
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  parent: search_graph::view::NodeRef<'a>,
  explore_bias: f64,
  rng: &mut R,
) -> Result<usize, UcbError>
where
  'a: 'b,
  G: 'a + Game,
  R: Rng,
{
  if graph.children(parent).next().is_none() {
    error!(
      "find_best_child_edge_index: no children for node {:?} with board: {:?}",
      parent,
      graph.node_state(parent),
    );
    return Err(UcbError::NoChildren);
  }
  let log_parent_visits = {
    let mut parent_visits = 0;
    for child in graph.children(parent) {
      parent_visits += graph.edge_data(child).statistics.as_payoff().visits();
    }
    if parent_visits == 0 {
      // When we visit a vertex for the first time, it will have zero visits.
      0.0
    } else {
      // Otherwise, it should be positive.
      f64::ln(parent_visits as f64)
    }
  };
  let mut best_index = 0;
  let mut best_ucb = ::std::f64::MIN;
  let mut sampling_count = 0u32;
  let ucb_iter = EdgeUcbIter::new(log_parent_visits, explore_bias, graph, graph.children(parent));
  for (index, ucb) in ucb_iter.enumerate() {
    match ucb {
      Ok(UcbSuccess::Select(_)) => {
        // trace!("find_best_child_edge_index: short-circuiting to select {}", index);
        // TODO: we should do tie-breaking here, too, but reading
        // through child edges in order helps a lot with debugging.
        return Ok(index);
      }
      Ok(UcbSuccess::Value(_, v)) => {
        match v.partial_cmp(&best_ucb) {
          None => {
            error!("find_best_child_edge_index: invalid floating-point comparison");
            return Err(UcbError::InvalidComputation);
          }
          Some(Ordering::Greater) => {
            // trace!("find_best_child_edge_index: new best index is {} with score {}", index, v);
            best_index = index;
            best_ucb = v;
            sampling_count = 1;
          }
          Some(Ordering::Equal) => {
            // We use reservoir sampling to break ties.
            // trace!("find_best_child_edge_index: found indices {} and {} with score {}; sampling to break tie", best_index, index, v);
            sampling_count += 1;
            if rng.gen_ratio(1, sampling_count) {
              best_index = index;
            }
            // trace!("find_best_child_edge_index: updated best index to {} after sampling", best_index);
          }
          _ => (),
        }
      }
      Err(e) => return Err(e),
    }
  }
  return Ok(best_index);
}
