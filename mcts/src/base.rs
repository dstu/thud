use ::thud_game;
use ::statistics::{NodeData, EdgeData};
use ::search_graph;

pub type ThudState = thud_game::state::State<thud_game::board::TranspositionalEquivalence>;
pub type ThudEdge<'a> = search_graph::nav::Edge<'a, ThudState, NodeData, EdgeData>;
pub type ThudGraph = search_graph::Graph<ThudState, NodeData, EdgeData>;
pub type ThudNode<'a> = search_graph::nav::Node<'a, ThudState, NodeData, EdgeData>;
pub type ThudChildList<'a> = search_graph::nav::ChildList<'a, ThudState, NodeData, EdgeData>;
pub type ThudParentList<'a> = search_graph::nav::ParentList<'a, ThudState, NodeData, EdgeData>;
pub type ThudParentListIter<'a> = search_graph::nav::ParentListIter<'a, ThudState, NodeData, EdgeData>;
pub type ThudMutNode<'a> = search_graph::mutators::MutNode<'a, ThudState, NodeData, EdgeData>;

