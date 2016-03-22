use ::mcts::base::{ChildList, Edge};
use ::game;

use ::rand::Rng;
use ::search_graph;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::result::Result;

#[derive(Debug)]
pub enum UcbSuccess {
    /// No value can be computed, but the UCB policy indicates that this child
    /// should be selected. E.g., the child has not yet been visited.
    Select,
    /// A value is computed.
    Value(f64),
}

#[derive(Debug)]
pub enum UcbError {
    /// There are no children to select from.
    NoChildren,
    /// An error was encountered during computation of UCB score (such as
    /// encountering a `None` result when `PartialCmp`'ing two floating-point
    /// values).
    InvalidComputation,
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

/// Returns the UCB policy result for the given values.
///
///  - `log_parent_visits`: logarithm of the count of parent visits
///  - `child_visits`: the count of child visits
///  - `child_payoff`: the accumulated score of payoffs seen when selecting child
///  - `explore_bias`: the UCB explore bias parameter
pub fn score(log_parent_visits: f64, child_visits: f64, child_payoff: f64, explore_bias: f64)
             -> Result<UcbSuccess, UcbError> {
    if child_visits == 0.0 {
        return Ok(UcbSuccess::Select)
    }
    Ok(UcbSuccess::Value(child_payoff / child_visits
                         + explore_bias * f64::sqrt(log_parent_visits / child_visits)))
}

/// Returns `true` iff `e` could be selected by the UCB policy during rollout
/// from its parent vertex.
pub fn is_best_child<'a>(e: &Edge<'a>, player: game::PlayerMarker, explore_bias: f64) -> bool {
    // TODO: this is totally broken because it assumes we haven't yet altered
    // the vertex statistics.
    let stats = e.get_data().statistics.get();
    if stats.visits == 0 {
        // Edge has been visited, but statistics aren't yet updated.
        trace!("edge {} is a best child because stats.visits == 0", e.get_id());
        return true
    }
    let parent = e.get_source();
    let siblings = parent.get_child_list();
    if siblings.len() == 1 {
        // Only child of parent.
        trace!("edge {} is a best child because it has no siblings", e.get_id());
        return true
    }
    trace!("edge {} (from node {}) has {} siblings", e.get_id(), parent.get_id(), siblings.len());
    let parent_stats = parent.get_data().statistics.get();
    let log_parent_visits = {
        let parent_visits = parent_stats.visits;
        if parent_visits == 0 {
            // We should never see this case, as the number of visits to the
            // parent vertex should never be less than the number of visits to a
            // child edge. But it's reasonable to fall back to a value of 0.0,
            // just in case.
            error!("edge {} (from node {}) has {} visits, but node {} has {} visits",
                   e.get_id(), parent.get_id(), stats.visits, parent.get_id(), parent_stats.visits);
            0.0
        } else {
            f64::ln(parent_visits as f64)
        }
    };
    let mut edge_ucb = None;
    let mut best_ucb = 0.0;
    // Scan through siblings to find the maximum UCB score. Several
    // short-circuit checks along the way help ameliorate this O(n) operation.
    for sibling_edge in siblings.iter() {
        match sibling_edge.get_target() {
            search_graph::Target::Unexpanded(_) => {
                // This sibling has not yet been visited. We know that this edge
                // has been visited, and we will always visit all siblings at
                // least once before returning to this one.
                trace!("edge {} is not a best child because it has an unexpanded sibling", e.get_id());
                return false
            },
            search_graph::Target::Expanded(_) => {
                let sibling_stats = sibling_edge.get_data().statistics.get();
                match score(log_parent_visits, sibling_stats.visits as f64,
                            stats.payoff.score(player) as f64, explore_bias) {
                    Ok(UcbSuccess::Select) => {
                        // Score computation short-circuits UCB to select this
                        // sibling, so we select it iff it is the edge we're
                        // considering. This only happens when the sibling
                        // hasn't been visited yet, and we checked for that
                        // above, but we handle the case again here to complete
                        // the match.
                        trace!("edge {} best child determined by weird edge case", e.get_id());
                        return sibling_edge.get_id() == e.get_id()
                    },
                    Ok(UcbSuccess::Value(ucb)) => {
                        trace!("is_best_child({:?}): edge {} (parent node {}) has ucb {}",
                               player, sibling_edge.get_id(), sibling_edge.get_source().get_id(), ucb);
                        if sibling_edge.get_id() == e.get_id() {
                            if best_ucb > ucb {
                                // We have already seen an edge with a greater
                                // UCB score.
                                trace!("edge {} is not a best child because we have seen a greater UCB score",
                                       e.get_id());
                                return false
                            }
                            edge_ucb = Some(ucb);
                        }
                        if let Some(u) = edge_ucb {
                            if ucb > u {
                                // This edge has a greater UCB score.
                                trace!("is_best_child({:?}): ucb of {} exceeds {} of edge",
                                       player, ucb, u);
                                return false
                            }
                        }
                        if ucb > best_ucb {
                            best_ucb = ucb;
                        }
                    },
                    Err(e) => panic!("error {:?} computing ucb for best child search", e),
                }
            },
        }
    }
    trace!("is_best_child: edge ucb of {:?} vs. best_ucb of {}", edge_ucb, best_ucb);
    match edge_ucb {
        Some(u) if u >= best_ucb =>
            // Target edge has a UCB score which matches the maximum we found.
            true,
        _ =>
            // Edges are not best children by default.
            false,
    }
}

pub fn find_best_child_edge<'a, R>(c: &ChildList<'a>, player: game::PlayerMarker, epoch: usize,
                                   explore_bias: f64, rng: &mut R) -> Result<usize, UcbError>
    where R: Rng {
        if c.is_empty() {
            return Err(UcbError::NoChildren)
        }
        let log_parent_visits =
            match c.get_source_node().get_data().statistics.get().visits {
                // When we visit a vertex for the first time, it will have zero visits.
                0 => 0.0,
                // Otherwise, it should be positive.
                parent_visits => f64::ln(parent_visits as f64),
            };
        let mut best_index = 0;
        let mut best_ucb = 0.0;
        let mut sampling_count = 0u32;
        for (index, child_edge) in c.iter().enumerate() {
            match child_edge.get_target() {
                search_graph::Target::Unexpanded(_) =>
                    // Unvisited children have top priority.
                    return Ok(index),
                search_graph::Target::Expanded(_) => {
                    let stats = child_edge.get_data().statistics.get();
                    match score(log_parent_visits, stats.visits as f64,
                                stats.payoff.score(player) as f64, explore_bias) {
                        Ok(UcbSuccess::Select) =>
                            // Score computation can short-circuit the decision.
                            return Ok(index),
                        Ok(UcbSuccess::Value(ucb)) => {
                            match ucb.partial_cmp(&best_ucb) {
                                None =>
                                    return Err(UcbError::InvalidComputation),
                                Some(Ordering::Greater) => {
                                    best_index = index;
                                    best_ucb = ucb;
                                    sampling_count = 1;
                                },
                                Some(Ordering::Equal) => {
                                    // We use reservoir sampling to break ties.
                                    sampling_count += 1;
                                    if rng.gen_weighted_bool(sampling_count) {
                                        best_index = index;
                                        best_ucb = ucb;
                                    }
                                },
                                _ => (),
                            }
                        },
                        Err(e) => return Err(e)

                    }
                },
            }
        }
        return Ok(best_index)
    }
