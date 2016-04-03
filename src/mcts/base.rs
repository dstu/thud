use ::game;
use ::mcts::statistics::{NodeData, EdgeData};
use ::search_graph;

pub type State = game::State<game::board::TranspositionalEquivalence>;
pub type Edge<'a> = search_graph::Edge<'a, State, NodeData, EdgeData>;
pub type MutEdge<'a> = search_graph::MutEdge<'a, State, NodeData, EdgeData>;
pub type Graph = search_graph::Graph<State, NodeData, EdgeData>;
pub type Node<'a> = search_graph::Node<'a, State, NodeData, EdgeData>;
pub type ChildList<'a> = search_graph::ChildList<'a, State, NodeData, EdgeData>;
pub type ChildListIter<'a> = search_graph::ParentListIter<'a, State, NodeData, EdgeData>;
pub type ParentList<'a> = search_graph::ParentList<'a, State, NodeData, EdgeData>;
pub type ParentListIter<'a> = search_graph::ParentListIter<'a, State, NodeData, EdgeData>;
pub type MutNode<'a> = search_graph::MutNode<'a, State, NodeData, EdgeData>;
pub type MutChildList<'a> = search_graph::MutChildList<'a, State, NodeData, EdgeData>;
pub type MutParentList<'a> = search_graph::MutParentList<'a, State, NodeData, EdgeData>;
pub type EdgeExpander<'a> = search_graph::EdgeExpander<'a, State, NodeData, EdgeData>;
pub type SearchPath<'a> = search_graph::SearchPath<'a, State, NodeData, EdgeData>;
