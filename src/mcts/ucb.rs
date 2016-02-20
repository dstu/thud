use ::mcts::base::{ChildList, Edge};
use ::game;

use ::rand::Rng;
use ::search_graph;

use std::cmp::Ordering;

#[derive(Debug)]
pub enum Result {
    Select,
    Value(f64),
    Err(Error),
}

#[derive(Debug)]
pub enum Error {
    NoChildren,
    InvalidComputation,
}

pub fn score(log_parent_visits: f64, child_visits: f64, child_payoff: f64, explore_bias: f64) -> Result {
    if child_visits == 0.0 {
        return Result::Select
    }
    Result::Value(child_payoff / child_visits
                  + explore_bias * f64::sqrt(log_parent_visits / child_visits))
}

pub fn is_best_child<'a>(e: &Edge<'a>, player: game::PlayerMarker, explore_bias: f64) -> bool {
    if e.get_data().statistics.visits == 0 {
        return true
    }
    let parent = e.get_source();
    let siblings = parent.get_child_list();
    if siblings.len() == 1 {
        return true
    }
    trace!("edge {} (from node {}) has {} siblings", e.get_id(), parent.get_id(), siblings.len());
    let log_parent_visits = {
        let parent_visits = parent.get_data().statistics.visits;
        if parent_visits == 0 {
            0.0
        } else {
            f64::ln(parent_visits as f64)
        }
    };
    let mut edge_ucb = None;
    let mut best_ucb = 0.0;
    for child_edge in siblings.iter() {
        match child_edge.get_target() {
            search_graph::Target::Unexpanded(_) => return false,
            search_graph::Target::Expanded(_) => {
                let stats = child_edge.get_data().statistics;
                match score(log_parent_visits, stats.visits as f64, stats.payoff.score(player) as f64, explore_bias) {
                    Result::Select => return child_edge.get_id() == e.get_id(),
                    Result::Value(ucb) => {
                        trace!("is_best_child({:?}): edge {} (parent node {}) has ucb {}", player, child_edge.get_id(), child_edge.get_source().get_id(), ucb);
                        if child_edge.get_id() == e.get_id() {
                            if best_ucb > ucb {
                                trace!("is_best_child({:?}): extant best ucb of {} exceeds {} of edge", player, best_ucb, ucb);
                                return false
                            }
                            edge_ucb = Some(ucb);
                        }
                        if let Some(u) = edge_ucb {
                            if ucb > u {
                                trace!("is_best_child({:?}): ucb of {} exceeds {} of edge", player, ucb, u);
                                return false
                            }
                        }
                        if ucb > best_ucb {
                            best_ucb = ucb;
                        }
                    },
                    Result::Err(e) => panic!("error {:?} computing ucb for best child search", e),
                }
            },
        }
    }
    trace!("is_best_child: edge ucb of {:?} vs. best_ucb of {}", edge_ucb, best_ucb);
    match edge_ucb {
        Some(u) if u >= best_ucb => true,
        _ => false,
    }
}

pub fn find_best_child_edge<'a, R: Rng>(c: &ChildList<'a>, player: game::PlayerMarker, explore_bias: f64, rng: &mut R)
                                        -> ::std::result::Result<usize, Error> {
    if c.is_empty() {
        return ::std::result::Result::Err(Error::NoChildren)
    }
    let log_parent_visits = {
        let parent_visits = c.get_source_node().get_data().statistics.visits;
        if parent_visits == 0 {
            0.0
        } else {
            f64::ln(parent_visits as f64)
        }
    };
    let children = c.iter().enumerate();
    let mut best_index = 0;
    let mut best_ucb = 0.0;
    for (index, child_edge) in children {
        match child_edge.get_target() {
            search_graph::Target::Unexpanded(_) => return ::std::result::Result::Ok(index),
            search_graph::Target::Expanded(_) => {
                let stats = &child_edge.get_data().statistics;
                match score(log_parent_visits, stats.visits as f64, stats.payoff.score(player) as f64, explore_bias) {
                    Result::Select => {
                        trace!("find_best_child_edge({:?}): edge {} is unvisited", player, child_edge.get_id());
                        return ::std::result::Result::Ok(index)
                    },
                    Result::Value(ucb) => {
                        trace!("find_best_child_edge({:?}): edge {} (parent node {}) has ucb {}", player, child_edge.get_id(), child_edge.get_source().get_id(), ucb);
                        match ucb.partial_cmp(&best_ucb) {
                            None => return ::std::result::Result::Err(Error::InvalidComputation),
                            Some(Ordering::Greater) => {
                                best_index = index;
                                best_ucb = ucb;
                            },
                            Some(Ordering::Equal) => {
                                if rng.gen_weighted_bool(2) {
                                    best_index = index;
                                    best_ucb = ucb;
                                }
                            },
                            _ => (),
                        }
                    },
                    Result::Err(e) => return ::std::result::Result::Err(e)

                }
            },
        }
    }
    return ::std::result::Result::Ok(best_index)
}
