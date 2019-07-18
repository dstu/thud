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
use search_graph::nav::{Edge, Node, ParentList};

/// Provides a method for selecting outgoing parent edges to follow during
/// backprop phase of MCTS.
pub trait BackpropSelector<'a, G, R>: for<'b> From<&'b SearchSettings>
where
  G: 'a + Game,
  R: Rng,
{
  type Items: Iterator<Item = Edge<'a, G::State, VertexData, EdgeData<G>>>;

  /// Returns the edges to follow when pushing statistics back up through the
  /// search graph.
  fn select(
    &self,
    parents: ParentList<'a, G::State, VertexData, EdgeData<G>>,
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
  node: Node<'a, G::State, VertexData, EdgeData<G>>,
  payoff: &'b G::Payoff,
  selector: &'b S,
  rng: &'b mut R,
) -> impl Iterator<Item = Edge<'a, G::State, VertexData, EdgeData<G>>> + 'b
where
  'a: 'b,
  G: Game,
  S: BackpropSelector<'a, G, R> + 'b,
  R: Rng,
{
  BackpropIter::new(node, payoff, selector, rng)
}

/// Chains together parent traversals.
struct BackpropIter<'a, 'b, G, S, R>
where
  G: Game,
  S: BackpropSelector<'a, G, R> + 'b,
  R: Rng,
{
  /// Nodes whose parent edges to traverse.
  stack: Vec<Node<'a, G::State, VertexData, EdgeData<G>>>,
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
    node: Node<'a, G::State, VertexData, EdgeData<G>>,
    payoff: &'b G::Payoff,
    selector: &'b S,
    rng: &'b mut R,
  ) -> Self {
    let parent_edges = selector.select(node.get_parent_list(), payoff, rng);
    BackpropIter {
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
  type Item = Edge<'a, G::State, VertexData, EdgeData<G>>;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(parent) = self.parent_edges.next() {
      if !parent.get_data().mark_backprop_traversal() {
        self.stack.push(parent.get_source());
        return Some(parent);
      }
    }
    while let Some(node) = self.stack.pop() {
      self.parent_edges = self
        .selector
        .select(node.get_parent_list(), self.payoff, self.rng);
      while let Some(parent) = self.parent_edges.next() {
        if !parent.get_data().mark_backprop_traversal() {
          self.stack.push(parent.get_source());
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
pub struct ParentSelectionIter<'a, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = Edge<'a, G::State, VertexData, EdgeData<G>>>,
{
  parents: I,
  explore_bias: f64,
  game_type: PhantomData<&'a G>,
}

impl<'a, G, I> ParentSelectionIter<'a, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = Edge<'a, G::State, VertexData, EdgeData<G>>>,
{
  pub fn new(parents: I, explore_bias: f64) -> Self {
    ParentSelectionIter {
      parents,
      explore_bias,
      game_type: PhantomData,
    }
  }
}

impl<'a, G, I> Iterator for ParentSelectionIter<'a, G, I>
where
  G: 'a + Game,
  I: Iterator<Item = Edge<'a, G::State, VertexData, EdgeData<G>>>,
{
  type Item = Edge<'a, G::State, VertexData, EdgeData<G>>;

  fn next(&mut self) -> Option<Edge<'a, G::State, VertexData, EdgeData<G>>> {
    loop {
      match self.parents.next() {
        None => return None,
        Some(e) => {
          if e.get_data().mark_backprop_traversal() {
            trace!(
              "ParentSelectionIter::next: edge {} (from node {}) was already visited in backtrace",
              e.get_id(),
              e.get_source().get_id()
            );
            continue;
          }
          if ucb::is_best_child::<G>(&e, self.explore_bias) {
            trace!(
              "ParentSelectionIter::next: edge {} (from node {}) is a best child",
              e.get_id(),
              e.get_source().get_id()
            );
            return Some(e);
          }
          trace!(
            "ParentSelectionIter::next: edge {} (data {:?}) is not a best child",
            e.get_id(),
            e.get_data()
          );
        }
      }
    }
  }
}

pub fn backprop<'a, 'b, G, S, R>(
  node: Node<'a, G::State, VertexData, EdgeData<G>>,
  payoff: &G::Payoff,
  selector: &S,
  rng: &mut R,
) where
  'a: 'b,
  G: Game,
  S: BackpropSelector<'a, G, R> + 'b,
  R: Rng,
{
  // Traverse parent nodes and place them into a materialized collection because
  // updating the statistics alters best child status, and backprop_iter returns
  // a lazy iterator.
  let statistics: Vec<&EdgeData<G>> = backprop_iter(node, payoff, selector, rng)
    .map(|edge| edge.get_data())
    .collect();
  for s in statistics {
    s.statistics.increment(payoff);
    s.mark_backprop_traversal();
  }
}
