//! Interface and implementations for backpropagation of game payoffs through a
//! search graph.

use std::iter::Iterator;
use std::marker::PhantomData;

use crate::game::{Game, Statistics};
use crate::graph::{EdgeData, VertexData};
use crate::ucb;
use crate::SearchSettings;
use log::trace;
use rand::Rng;

/// Provides a method for selecting outgoing parent edges to follow during
/// backprop phase of MCTS.
pub trait BackpropSelector<'a, G: Game, R: Rng>: for<'b> From<&'b SearchSettings> {
  type Items: Iterator<Item = search_graph::view::EdgeRef<'a>>;

  /// Returns the edges to follow when pushing statistics back up through the
  /// search graph.
  fn select<I: IntoIterator<Item = search_graph::view::EdgeRef<'a>>>(
    &self,
    graph: &search_graph::view::View<G::State, VertexData, EdgeData<G>>,
    parents: I,
    payoff: &G::Payoff,
    rng: &mut R,
  ) -> Self::Items;
}

/// Returns an iterator that traverses the game graph upwards from `node` up to
/// the root vertices of the search graph. The caller may then update the
/// statistics of each edge produced by this iterator. Keep in mind that the
/// iterator is lazy, and the edges that it yields may be affected by statistics
/// updates if they are applied during iteration.
pub fn backprop_iter<'a, 'b, G, S, R>(
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  node: search_graph::view::NodeRef<'a>,
  payoff: &'b G::Payoff,
  selector: &'b S,
  rng: &'b mut R,
) -> impl Iterator<Item = search_graph::view::EdgeRef<'a>> + 'b
where
  'a: 'b,
  G: Game,
  S: BackpropSelector<'a, G, R>,
  R: Rng,
{
  BackpropIter::new(graph, node, payoff, selector, rng)
}

/// Chains together parent traversals.
struct BackpropIter<'a, 'b, G, S, R>
where
  G: Game,
  S: BackpropSelector<'a, G, R> + 'b,
  R: Rng,
{
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  /// Nodes whose parent edges to traverse.
  stack: Vec<search_graph::view::NodeRef<'a>>,
  /// Edges from most recently examined node.
  parent_edges: S::Items,
  payoff: &'b G::Payoff,
  selector: &'b S,
  rng: &'b mut R,
}

impl<'a, 'b, G, S, R> BackpropIter<'a, 'b, G, S, R>
where
  G: Game,
  S: BackpropSelector<'a, G, R>,
  R: Rng,
{
  fn new(
    graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
    node: search_graph::view::NodeRef<'a>,
    payoff: &'b G::Payoff,
    selector: &'b S,
    rng: &'b mut R,
  ) -> Self {
    let parent_edges = selector.select(graph, graph.parents(node), payoff, rng);
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

impl<'a, 'b, G, S, R> Iterator for BackpropIter<'a, 'b, G, S, R>
where
  G: Game,
  S: BackpropSelector<'a, G, R>,
  R: Rng,
{
  type Item = search_graph::view::EdgeRef<'a>;

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
          .select(self.graph, self.graph.parents(node), self.payoff, self.rng);
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
pub struct ParentSelectionIter<'a, 'b, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'a>>,
{
  graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  parents: I,
  explore_bias: f64,
  game_type: PhantomData<&'a G>,
}

impl<'a, 'b, G, I> ParentSelectionIter<'a, 'b, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'a>>,
{
  pub fn new(
    graph: &'b search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
    parents: I,
    explore_bias: f64,
  ) -> Self {
    ParentSelectionIter {
      graph,
      parents,
      explore_bias,
      game_type: PhantomData,
    }
  }
}

impl<'a, 'b, G, I> Iterator for ParentSelectionIter<'a, 'b, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = search_graph::view::EdgeRef<'a>>,
{
  type Item = search_graph::view::EdgeRef<'a>;

  fn next(&mut self) -> Option<search_graph::view::EdgeRef<'a>> {
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

pub fn backprop<'a, 'b, G, S, R>(
  graph: &search_graph::view::View<'a, G::State, VertexData, EdgeData<G>>,
  node: search_graph::view::NodeRef<'a>,
  payoff: &G::Payoff,
  selector: &S,
  rng: &mut R,
) where
  'a: 'b,
  G: Game + 'a,
  S: BackpropSelector<'a, G, R> + 'b,
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
