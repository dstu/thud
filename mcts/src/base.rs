use ::thud_game;
use ::{EdgeData, VertexData};
use ::search_graph;
use ::statistics::ThudStatistics;

pub type ThudState = thud_game::state::State<thud_game::board::TranspositionalEquivalence>;
pub type ThudEdge<'a> = search_graph::nav::Edge<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudGraph = search_graph::Graph<ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudNode<'a> = search_graph::nav::Node<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudChildList<'a> = search_graph::nav::ChildList<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudParentList<'a> = search_graph::nav::ParentList<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudParentListIter<'a> = search_graph::nav::ParentListIter<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;
pub type ThudMutNode<'a> = search_graph::mutators::MutNode<'a, ThudState, VertexData, EdgeData<ThudStatistics, thud_game::Action>>;

