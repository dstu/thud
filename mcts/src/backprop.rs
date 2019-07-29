//! Interface and implementations for backpropagation of game payoffs through a
//! search graph.

use std::iter::Iterator;

use crate::game::{Game, Statistics};
use crate::graph::{EdgeData, VertexData};
use crate::ucb;
use crate::SearchSettings;
use log::trace;
use rand::Rng;

/// Provides a method for selecting incoming parent edges to follow during
/// backprop phase of MCTS.
pub trait BackpropSelector<'id>: for<'a> From<&'a SearchSettings> {
  type Items: Iterator<Item = search_graph::view::EdgeRef<'id>>;

  /// Returns the edges to follow when pushing statistics back up through the
  /// search graph.
  fn select<G: Game, R: Rng>(
    &self,
    graph: &search_graph::view::View<'_, 'id, G::State, VertexData, EdgeData<G>>,
    node: search_graph::view::NodeRef<'id>,
    payoff: &G::Payoff,
    rng: &mut R,
  ) -> Self::Items;
}

pub struct FirstParentSelector {}

impl<'a> From<&'a SearchSettings> for FirstParentSelector {
  fn from(_: &'a SearchSettings) -> Self {
    FirstParentSelector {}
  }
}

impl<'id> BackpropSelector<'id> for FirstParentSelector {
  type Items = std::option::IntoIter<search_graph::view::EdgeRef<'id>>;

  fn select<G: Game, R: Rng>(
    &self,
    graph: &search_graph::view::View<'_, 'id, G::State, VertexData, EdgeData<G>>,
    node: search_graph::view::NodeRef<'id>,
    _payoff: &G::Payoff,
    _rng: &mut R,
  ) -> Self::Items {
    graph.parents(node).next().into_iter()
  }
}

/// Returns an iterator that traverses the game graph upwards from `node` up to
/// the root vertices of the search graph. The caller may then update the
/// statistics of each edge produced by this iterator. Keep in mind that the
/// iterator is lazy, and the edges that it yields may be affected by statistics
/// updates if they are applied during iteration.
pub fn backprop_iter<'a, 'b, 'id, G, S, R>(
  graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  node: search_graph::view::NodeRef<'id>,
  payoff: &'b G::Payoff,
  selector: &'b S,
  rng: &'b mut R,
) -> impl Iterator<Item = search_graph::view::EdgeRef<'id>> + 'b
where
  'a: 'b,
  G: Game,
  S: BackpropSelector<'id>,
  R: Rng,
{
  BackpropIter::new(graph, node, payoff, selector, rng)
}

/// Chains together parent traversals.
struct BackpropIter<'a, 'b, 'id, G, S, R>
where
  G: Game,
  S: BackpropSelector<'id> + 'b,
  R: Rng,
{
  graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  /// Nodes whose parent edges to traverse.
  stack: Vec<search_graph::view::NodeRef<'id>>,
  /// Edges from most recently examined node.
  parent_edges: S::Items,
  payoff: &'b G::Payoff,
  selector: &'b S,
  rng: &'b mut R,
}

impl<'a, 'b, 'id, G, S, R> BackpropIter<'a, 'b, 'id, G, S, R>
where
  G: Game,
  S: BackpropSelector<'id> + 'b,
  R: Rng,
{
  fn new(
    graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
    node: search_graph::view::NodeRef<'id>,
    payoff: &'b G::Payoff,
    selector: &'b S,
    rng: &'b mut R,
  ) -> Self {
    let parent_edges = selector.select(graph, node, payoff, rng);
    BackpropIter {
      graph,
      stack: vec![],
      parent_edges,
      payoff,
      selector,
      rng,
    }
  }
}

impl<'a, 'b, 'id, G, S, R> Iterator for BackpropIter<'a, 'b, 'id, G, S, R>
where
  G: Game,
  S: BackpropSelector<'id> + 'b,
  R: Rng,
{
  type Item = search_graph::view::EdgeRef<'id>;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(parent) = self.parent_edges.next() {
      if !self.graph.edge_data(parent).mark_backprop_traversal() {
        self.stack.push(self.graph.edge_source(parent));
        return Some(parent);
      }
    }
    while let Some(node) = self.stack.pop() {
      self.parent_edges =
        self
          .selector
          .select(self.graph, node, self.payoff, self.rng);
      while let Some(parent) = self.parent_edges.next() {
        if !self.graph.edge_data(parent).mark_backprop_traversal() {
          self.stack.push(self.graph.edge_source(parent));
          return Some(parent);
        }
      }
    }
    None
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.parent_edges.size_hint().0, None)
  }
}

/// Iterable view over parents of a graph node that selects parents for which
/// this node is a best child.
pub struct ParentSelectionIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  parents: I,
  explore_bias: f64,
}

impl<'a, 'b, 'id, G, I> ParentSelectionIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  pub fn new(
    graph: &'b search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
    parents: I,
    explore_bias: f64,
  ) -> Self {
    ParentSelectionIter {
      graph,
      parents,
      explore_bias,
    }
  }
}

impl<'a, 'b, 'id, G, I> Iterator for ParentSelectionIter<'a, 'b, 'id, G, I>
where
  G: Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'id>>,
{
  type Item = search_graph::view::EdgeRef<'id>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.parents.next() {
        None => return None,
        Some(e) => {
          if self.graph.edge_data(e).mark_backprop_traversal() {
            trace!("ParentSelectionIter::next: edge was already visited in backtrace",);
            continue;
          }
          if ucb::is_best_child::<G>(self.graph, e, self.explore_bias) {
            trace!(
              "ParentSelectionIter::next: edge {:?} (from node {:?}) is a best child",
              e,
              self.graph.edge_source(e),
            );
            return Some(e);
          }
          trace!(
            "ParentSelectionIter::next: edge {:?} (data {:?}) is not a best child",
            e,
            self.graph.edge_data(e),
          );
        }
      }
    }
  }
}

pub fn backprop<'a, 'id, G, S, R>(
  graph: &search_graph::view::View<'a, 'id, G::State, VertexData, EdgeData<G>>,
  node: search_graph::view::NodeRef<'id>,
  payoff: &G::Payoff,
  selector: &S,
  rng: &mut R,
) where
  G: Game,
  S: BackpropSelector<'id>,
  R: Rng,
{
  // Traverse parent nodes and place them into a materialized collection because
  // updating the statistics alters best child status, and backprop_iter returns
  // a lazy iterator.
  let statistics: Vec<&EdgeData<G>> = backprop_iter(graph, node, payoff, selector, rng)
    .map(|edge| graph.edge_data(edge))
    .collect();
  for s in statistics {
    s.statistics.increment(payoff);
    s.mark_backprop_traversal();
  }
}
