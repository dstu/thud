use ::thud_game;
use ::statistics::{NodeData, EdgeData};
use ::search_graph;

pub type State = thud_game::state::State<thud_game::board::TranspositionalEquivalence>;
pub type Edge<'a> = search_graph::nav::Edge<'a, State, NodeData, EdgeData>;
pub type MutEdge<'a> = search_graph::mutators::MutEdge<'a, State, NodeData, EdgeData>;
pub type Graph = search_graph::Graph<State, NodeData, EdgeData>;
pub type Node<'a> = search_graph::nav::Node<'a, State, NodeData, EdgeData>;
pub type ChildList<'a> = search_graph::nav::ChildList<'a, State, NodeData, EdgeData>;
pub type ChildListIter<'a> = search_graph::nav::ParentListIter<'a, State, NodeData, EdgeData>;
pub type ParentList<'a> = search_graph::nav::ParentList<'a, State, NodeData, EdgeData>;
pub type ParentListIter<'a> = search_graph::nav::ParentListIter<'a, State, NodeData, EdgeData>;
pub type MutNode<'a> = search_graph::mutators::MutNode<'a, State, NodeData, EdgeData>;
pub type MutChildList<'a> = search_graph::mutators::MutChildList<'a, State, NodeData, EdgeData>;
pub type MutParentList<'a> = search_graph::mutators::MutParentList<'a, State, NodeData, EdgeData>;
pub type SearchStack<'a> = search_graph::search::Stack<'a, State, NodeData, EdgeData>;
