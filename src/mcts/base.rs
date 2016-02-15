use ::game;
use ::mcts::statistics::{NodeData, EdgeData};
use ::search_graph;

pub type Edge<'a> = search_graph::Edge<'a, game::State, NodeData, EdgeData>;
pub type MutEdge<'a> = search_graph::MutEdge<'a, game::State, NodeData, EdgeData>;
pub type Graph = search_graph::Graph<game::State, NodeData, EdgeData>;
pub type Node<'a> = search_graph::Node<'a, game::State, NodeData, EdgeData>;
pub type ChildList<'a> = search_graph::ChildList<'a, game::State, NodeData, EdgeData>;
pub type ParentList<'a> = search_graph::ParentList<'a, game::State, NodeData, EdgeData>;
pub type MutNode<'a> = search_graph::MutNode<'a, game::State, NodeData, EdgeData>;
pub type MutChildList<'a> = search_graph::MutChildList<'a, game::State, NodeData, EdgeData>;
pub type MutParentList<'a> = search_graph::MutParentList<'a, game::State, NodeData, EdgeData>;
pub type EdgeExpander<'a> = search_graph::EdgeExpander<'a, game::State, NodeData, EdgeData>;
