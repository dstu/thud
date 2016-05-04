mod edge;
mod traversals;
mod vertex;

pub use edge::EdgeData;
pub use traversals::{AtomicTraversals, Traversals};
pub use vertex::VertexData;

// pub type Graph<G: Game> =
//     search_graph::Graph<G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type Vertex<'a, G: 'a + Game> =
//     search_graph::nav::Node<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type Edge<'a, G: 'a + Game> =
//     search_graph::nav::Edge<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ChildList<'a, G: 'a + Game> =
//     search_graph::nav::ChildList<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ChildListIter<'a, G: 'a + Game> =
//     search_graph::nav::ChildListIter<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ParentList<'a, G: 'a + Game> =
//     search_graph::nav::ParentList<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type ParentListIter<'a, G: 'a + Game> =
//     search_graph::nav::ParentListIter<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
// pub type MutVertex<'a, G: 'a + Game> =
//     search_graph::mutators::MutNode<'a, G::State, VertexData, EdgeData<G::Statistics, G::Action>>;
