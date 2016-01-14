use std::fmt;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;

/// Internal derived type for edge IDs.
///
/// This type is not exposed publicly because it does not identify the graph
/// that it belongs to, which makes it only slightly less dangerous than a
/// pointer with no lifetime.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ArcId(usize);

impl ArcId {
    /// Converts an `ArcId` to a usize to index into a collection.
    fn as_usize(self) -> usize {
        let ArcId(value) = self;
        value
    }
}

/// Internal derived type for vertex IDs.
///
/// For a given graph, distinct `StateId`s are associated with distinct game
/// states. This type is not exposed publicly because it does not identify the
/// graph that it belongs to, which makes it only slightly less dangerous than a
/// pointer with no lifetime.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateId(usize);

impl StateId {
    /// Converts a `StateId` to a usize to index into a collection.
    fn as_usize(self) -> usize {
        let StateId(value) = self;
        value
    }
}

/// Retains a mapping from game states to distinct IDs.
///
/// The game state type `T` is required to derive from `Clone` to accommodate a
/// limitation of the `HashMap` interface.
struct StateNamespace<T> where T: Hash + Eq + Clone {
    states: HashMap<T, StateId>,
}

/// The result of inserting a game state into a `StateNamespace`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum NamespaceInsertion {
    /// State was already present, with the given ID.
    Present(StateId),
    /// State was new, and is inserted with the given ID.
    New(StateId),
}

impl<T> StateNamespace<T> where T: Hash + Eq + Clone {
    /// Creates a new, empty `StateNamespace`.
    fn new() -> Self {
        StateNamespace {
            states: HashMap::new(),
        }
    }

    /// Retrieves a `StateId` for `state`, creating a new one if necessary.
    ///
    /// This may insert a new state into `self` or simply retrieve the `StateId`
    /// associated with it in a prior insertion operation.
    fn get_or_insert(&mut self, state: T) -> NamespaceInsertion {
        let next_state_id = StateId(self.states.len());
        match self.states.entry(state) {
            Entry::Occupied(e) => NamespaceInsertion::Present(*e.get()),
            Entry::Vacant(e) => NamespaceInsertion::New(*e.insert(next_state_id)),
        }
    }

    /// Retrieves a `StateId` for `state`.
    ///
    /// If `state` has not been inserted, returns `None`.
    fn get(&self, state: &T) -> Option<StateId> {
        self.states.get(state).map(|x| *x)
    }
}

/// The target of an outgoing graph edge.
///
/// A search graph is built up incrementally. Any vertices in its initial state
/// are typically added with all of their edges in the unexpanded
/// state. Graph-modifying operations which are executed while exploring the
/// game state topology will expand these edges. Cycle detection is done at edge
/// expansion time.
pub enum Target<T, R> {
    /// Edge has not yet been expanded.
    Unexpanded(R),
    /// Edge has been expanded but leads to a cycle. Because cycle detection is
    /// done at edge expansion time, this usually means that another edge, which
    /// was expanded previously, has the value `Target::Expanded` and points to
    /// the same vertex.
    Cycle(T),
    /// Edge has been expanded and was the expanded edge that lead to the game
    /// state which it points to.
    Expanded(T),
}

impl<T, R> Clone for Target<T, R> where T: Clone, R: Clone {
    fn clone(&self) -> Self {
        match self {
            &Target::Cycle(ref t) => Target::Cycle(t.clone()),
            &Target::Unexpanded(ref r) => Target::Unexpanded(r.clone()),
            &Target::Expanded(ref t) => Target::Expanded(t.clone()),
        }
    }
}

impl<T, R> Copy for Target<T, R> where T: Copy, R: Copy { }

impl<T, R> fmt::Debug for Target<T, R> where T: fmt::Debug, R: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Target::Cycle(ref t) => write!(f, "Cycle({:?})", t),
            Target::Unexpanded(ref r) => write!(f, "Unexpanded({:?})", r),
            Target::Expanded(ref t) => write!(f, "Expanded({:?})", t),
        }
    }
}

/// Internal type for graph vertices.
#[derive(Debug)]
struct Vertex<S> where S: Debug {
    /// Vertex data.
    data: S,
    /// Parent edges pointing into this vertex.
    parents: Vec<ArcId>,
    /// Child edges pointing out of this vertex.
    children: Vec<ArcId>,
}

/// Internal type for graph edges.
#[derive(Debug)]
struct Arc<A> where A: Debug {
    /// Edge data.
    data: A,
    /// Source vertex.
    source: StateId,
    /// Target vertex. If this arc is unexpanded, it is
    /// `Target::Unexpanded(())`; otherwise, it is either `Target::Cycle(id)` or
    /// `Target::Expanded(id)` for target vertex with a `StateId` of `id`.
    target: Target<StateId, ()>,
}

/// A search graph.
///
/// Supports incremental rollout of game state topology, vertex de-duplication
/// with transposition tables, and cycle detection. Does not support deletion.
///
/// - `T`: The type of game states. It is required to derive `Hash` and `Eq` to
///   so that it may be stored in a hashtable, where game states are looked up to
///   support de-duplication of game states. It is required to derive `Clone` to
///   accommodate a limitation of the `HashMap` interface.
/// - `S`: The type of graph vertex data.
/// - `A`: The type of graph edge data.
pub struct Graph<T, S, A> where T: Hash + Eq + Clone, S: Debug, A: Debug {
    /// Lookup table that maps from game states to `StateId`.
    state_ids: StateNamespace<T>,
    vertices: Vec<Vertex<S>>,  // Indexed by StateId.
    arcs: Vec<Arc<A>>,  // Indexed by ArcId.
}

impl<T, S, A> Graph<T, S, A> where T: Hash + Eq + Clone, S: Debug, A: Debug {
    /// Creates an empty `Graph` with no vertices or edges.
    pub fn new() -> Self {
        Graph {
            state_ids: StateNamespace::new(),
            vertices: Vec::new(),
            arcs: Vec::new(),
        }
    }

    /// Returns the vertex for the given `StateId`.
    fn get_vertex(&self, state: StateId) -> &Vertex<S> {
        &self.vertices[state.as_usize()]
    }

    /// Returns the vertex for the given `StateId`.
    fn get_vertex_mut(&mut self, state: StateId) -> &mut Vertex<S> {
        &mut self.vertices[state.as_usize()]
    }

    /// Returns the edge for the given `ArcId`.
    fn get_arc(&self, arc: ArcId) -> &Arc<A> {
        &self.arcs[arc.as_usize()]
    }

    /// Returns the edge for the given `ArcId`.
    fn get_arc_mut(&mut self, arc: ArcId) -> &mut Arc<A> {
        &mut self.arcs[arc.as_usize()]
    }

    /// Adds a new vertex with the given data, returning a mutable reference to it.
    ///
    /// This method does not add incoming or outgoing edges (expanded or
    /// not). That must be done by calling `add_arc` with the new vertex
    /// `StateId`.
    fn add_vertex(&mut self, data: S) -> &mut Vertex<S> {
        self.vertices.push(Vertex { data: data, parents: Vec::new(), children: Vec::new() });
        self.vertices.last_mut().unwrap()
    }

    /// Adds a new edge with the given data, source, and target.
    ///
    /// Iff `target` is `Target::Expanded(id)`, the vertex with `StateId` of
    /// `id` will have the vertex `source` added as a parent.
    fn add_arc(&mut self, data: A, source: StateId, target: Target<StateId, ()>) {
        let arc = Arc { data: data, source: source, target: target, };
        let arc_id = ArcId(self.arcs.len());
        if let Target::Expanded(target_id) = target {
            self.get_vertex_mut(target_id).parents.push(arc_id);
        }
        self.get_vertex_mut(source).children.push(arc_id);
        self.arcs.push(arc);
    }

    /// Checks whether a path exists from `source` to `target`.
    ///
    /// Paths are found using a simple depth-first search. This method only
    /// follows arcs with destination type `Target::Expanded`, so it does not
    /// find paths that go through an ancestor of `source`.
    fn path_exists(&self, source: StateId, target: StateId) -> bool {
        let mut frontier = vec![source];
        while !frontier.is_empty() {
            let state = frontier.pop().unwrap();
            if target == state {
                return true
            }
            for arc_id in &self.get_vertex(state).children {
                let arc = self.get_arc(*arc_id);
                if let Target::Expanded(target_id) = arc.target {
                    frontier.push(target_id);
                }
            }
        }
        false
    }

    /// Gets a node handle for the given game state.
    ///
    /// If `state` does not correspond to a known game state, returns `None`.
    pub fn get_node<'s>(&'s self, state: &T) -> Option<Node<'s, T, S, A>> {
        match self.state_ids.get(&state) {
            Some(id) => Some(Node { graph: self, id: id, }),
            None => None,
        }
    }

    /// Gets a mutable node handle for the given game state.
    ///
    /// If `state` does not correspond to a known game state, returns `None`.
    pub fn get_node_mut<'s>(&'s mut self, state: &T) -> Option<MutNode<'s, T, S, A>> {
        match self.state_ids.get(state) {
            Some(id) => Some(MutNode { graph: self, id: id, }),
            None => None,
        }
    }

    /// Adds a root vertex (one with no parents) for the given game state and
    /// data and returns a mutable handle for it.
    ///
    /// If `state` is already known, returns a mutable handle to that state,
    /// ignoring the `data` parameter. As a result, this method is guaranteed to
    /// return a handle for a root vertex only when `state` is a novel game
    /// state.
    pub fn add_root<'s>(&'s mut self, state: T, data: S) -> MutNode<'s, T, S, A> {
        let node_id = match self.state_ids.get_or_insert(state) {
            NamespaceInsertion::Present(id) => id,
            NamespaceInsertion::New(id) => {
                self.add_vertex(data);
                id
            },
        };
        MutNode { graph: self, id: node_id, }
    }
}

/// Immutable handle to a graph vertex, called a "node handle."
///
/// This zipper-like type enables traversal of a graph along the vertex's
/// incoming and outgoing edges.
pub struct Node<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> Node<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn children(&self) -> &'a [ArcId] {
        &self.graph.get_vertex(self.id).children
    }

    /// Returns an immutable ID that is guaranteed to identify this vertex
    /// uniquely within its graph.
    pub fn get_id(&self) -> usize {
        self.id.as_usize()
    }

    fn parents(&self) -> &'a [ArcId] {
        &self.graph.get_vertex(self.id).parents
    }

    pub fn get_data(&self) -> &'a S {
        &self.graph.get_vertex(self.id).data
    }

    /// Returns true iff this vertex has no outgoing edges (regardless of
    /// whether they are expanded).
    pub fn is_leaf(&self) -> bool {
        self.children().is_empty()
    }

    /// Returns true iff this vertex has no incoming edges.
    pub fn is_root(&self) -> bool {
        self.parents().is_empty()
    }

    /// Returns a traversible list of outgoing edges.
    pub fn get_child_list(&self) -> ChildList<'a, T, S, A> {
        ChildList { graph: self.graph, id: self.id, }
    }

    /// Returns a traversible list of incoming edges.
    pub fn get_parent_list(&self) -> ParentList<'a, T, S, A> {
        ParentList { graph: self.graph, id: self.id, }
    }
}

/// A traversible list of a vertex's outgoing edges.
pub struct ChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> ChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    /// Returns the number of edges.
    pub fn len(&self) -> usize {
        self.vertex().children.len()
    }

    /// Returns true iff there are no outgoing edges.
    pub fn is_empty(&self) -> bool {
        self.vertex().children.is_empty()
    }

    /// Returns a node handle for the vertex these edges originate from.
    pub fn get_source_node(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.id, }
    }

    /// Returns an edge handle for the `i`th edge.
    pub fn get_edge(&self, i: usize) -> Edge<'a, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().children[i], }
    }
}

/// A traversible list of a vertex's incoming edges.
pub struct ParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> ParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    /// Returns the number of edges.
    pub fn len(&self) -> usize {
        self.vertex().parents.len()
    }

    /// Returns true iff there are no incoming edges.
    pub fn is_empty(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    /// Returns a node handle for the vertex these edges point to.
    pub fn target_node(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.id, }
    }

    /// Returns an edge handle for the `i`th edge.
    pub fn get_edge(&self, i: usize) -> Edge<'a, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().parents[i] }
    }
}

/// Immutable handle to a graph edge, called an "edge handle."
///
/// This zipper-like type enables traversal of a graph along the edge's source
/// and target vertices.
pub struct Edge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: ArcId,
}

impl<'a, T, S, A> Edge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn arc(&self) -> &'a Arc<A> {
        self.graph.get_arc(self.id)
    }

    /// Returns an immutable ID that is guaranteed to identify this edge
    /// uniquely within its graph.
    pub fn get_id(&self) -> usize {
        self.id.as_usize()
    }

    pub fn get_data(&self) -> &'a A {
        &self.arc().data
    }

    /// Returns a node handle for this edge's source vertex.
    pub fn get_source(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.arc().source, }
    }

    /// Returns a node handle for this edge's target vertex.
    pub fn get_target(&self) -> Target<Node<'a, T, S, A>, ()> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(Node { graph: self.graph, id: id, }),
            Target::Unexpanded(_) => Target::Unexpanded(()),
            Target::Expanded(id) => Target::Expanded(Node { graph: self.graph, id: id, }),
        }
    }
}

pub struct MutNode<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> MutNode<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex<'s>(&'s self) -> &'s Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    fn vertex_mut<'s>(&'s mut self) -> &'s mut Vertex<S> {
        self.graph.get_vertex_mut(self.id)
    }

    pub fn get_id(&self) -> usize {
        self.id.as_usize()
    }

    pub fn get_data<'s>(&'s self) -> &'s S {
        &self.vertex().data
    }

    pub fn get_data_mut<'s>(&'s mut self) -> &'s mut S {
        &mut self.vertex_mut().data
    }

    pub fn is_leaf(&self) -> bool {
        self.vertex().children.is_empty()
    }

    pub fn is_root(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    pub fn get_child_list<'s>(&'s self) -> ChildList<'s, T, S, A> {
        ChildList { graph: self.graph, id: self.id, }
    }

    pub fn get_child_list_mut<'s>(&'s mut self) -> MutChildList<'s, T, S, A> {
        MutChildList { graph: self.graph, id: self.id, }
    }

    pub fn to_child_list(self) -> MutChildList<'a, T, S, A> {
        MutChildList { graph: self.graph, id: self.id, }
    }

    pub fn get_parent_list<'s>(&'s self) -> ParentList<'s, T, S, A> {
        ParentList { graph: self.graph, id: self.id, }
    }

    pub fn get_parent_list_mut<'s>(&'s mut self) -> MutParentList<'s, T, S, A> {
        MutParentList { graph: self.graph, id: self.id, }
    }

    pub fn to_parent_list(self) -> MutParentList<'a, T, S, A> {
        MutParentList { graph: self.graph, id: self.id, }
    }

    pub fn get_child_adder<'s>(&'s mut self) -> EdgeAdder<'s, T, S, A> {
        EdgeAdder { graph: self.graph, id: self.id, }
    }

    pub fn to_child_adder(self) -> EdgeAdder<'a, T, S, A> {
        EdgeAdder { graph: self.graph, id: self.id, }
    }
}

pub struct MutChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> MutChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex<'s>(&'s self) -> &'s Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().children.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertex().children.is_empty()
    }

    pub fn get_edge<'s>(&'s self, i: usize) -> Edge<'s, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().children[i], }
    }

    pub fn get_edge_mut<'s>(&'s mut self, i: usize) -> MutEdge<'s, T, S, A> {
        let id = self.vertex().children[i];
        MutEdge { graph: self.graph, id: id, }
    }

    pub fn to_edge(self, i: usize) -> MutEdge<'a, T, S, A> {
        let id = self.vertex().children[i];
        MutEdge { graph: self.graph, id: id, }
    }

    pub fn get_source_node<'s>(&'s self) -> Node<'s, T, S, A> {
        Node { graph: self.graph, id: self.id, }
    }

    pub fn get_source_node_mut<'s>(&'s mut self) -> MutNode<'s, T, S, A> {
        MutNode { graph: self.graph, id: self.id, }
    }

    pub fn to_source_node(self) -> MutNode<'a, T, S, A> {
        MutNode { graph: self.graph, id: self.id, }
    }
}

pub struct MutParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> MutParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex<'s>(&'s self) -> &'s Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    pub fn get_edge<'s>(&'s self, i: usize) -> Edge<'s, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().parents[i], }
    }

    pub fn get_edge_mut<'s>(&'s mut self, i: usize) -> MutEdge<'s, T, S, A> {
        let id = self.vertex().parents[i];
        MutEdge { graph: self.graph, id: id, }
    }

    pub fn to_edge(self, i: usize) -> MutEdge<'a, T, S, A> {
        let id = self.vertex().parents[i];
        MutEdge { graph: self.graph, id: id, }
    }
}

pub struct MutEdge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: ArcId,
}

impl<'a, T, S, A> MutEdge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn arc(&self) -> &Arc<A> {
        self.graph.get_arc(self.id)
    }

    fn arc_mut(&mut self) -> &mut Arc<A> {
        self.graph.get_arc_mut(self.id)
    }

    pub fn get_id(&self) -> usize {
        self.id.as_usize()
    }

    pub fn get_data(&self) -> &A {
        &self.arc().data
    }

    pub fn get_data_mut(&mut self) -> &mut A {
        &mut self.arc_mut().data
    }

    pub fn get_target<'s>(&'s self) -> Target<Node<'s, T, S, A>, ()> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(Node { graph: self.graph, id: id, }),
            Target::Unexpanded(_) => Target::Unexpanded(()),
            Target::Expanded(id) =>
                Target::Expanded(Node { graph: self.graph, id: id, }),
        }
    }

    pub fn get_target_mut<'s>(&'s mut self) -> Target<MutNode<'s, T, S, A>, EdgeExpander<'s, T, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(MutNode { graph: self.graph, id: id, }),
            Target::Unexpanded(_) => Target::Unexpanded(EdgeExpander { graph: self.graph, id: self.id, }),
            Target::Expanded(id) =>
                Target::Expanded(MutNode { graph: self.graph, id: id, }),
        }
    }

    pub fn to_target(self) -> Target<MutNode<'a, T, S, A>, EdgeExpander<'a, T, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(MutNode { graph: self.graph, id: id, }),
            Target::Unexpanded(_) => Target::Unexpanded(EdgeExpander { graph: self.graph, id: self.id, }),
            Target::Expanded(id) =>
                Target::Expanded(MutNode { graph: self.graph, id: id, }),
        }
    }

    pub fn get_source<'s>(&'s self) -> Node<'s, T, S, A> {
        Node { graph: self.graph, id: self.arc().source, }
    }

    pub fn get_source_mut<'s>(&'s mut self) -> MutNode<'s, T, S, A> {
        let id = self.arc().source;
        MutNode { graph: self.graph, id: id, }
    }

    pub fn to_source(self) -> MutNode<'a, T, S, A> {
        let id = self.arc().source;
        MutNode { graph: self.graph, id: id, }
    }
}

pub struct EdgeExpander<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: ArcId,
}

impl<'a, T, S, A> EdgeExpander<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn arc(&self) -> &Arc<A> {
        self.graph.get_arc(self.id)
    }

    fn arc_mut(&mut self) -> &mut Arc<A> {
        self.graph.get_arc_mut(self.id)
    }

    pub fn get_edge<'s>(&'s self) -> Edge<'s, T, S, A> {
        Edge { graph: self.graph, id: self.id, }
    }

    pub fn get_edge_mut<'s>(&'s mut self) -> MutEdge<'s, T, S, A> {
        MutEdge { graph: self.graph, id: self.id, }
    }

    pub fn to_edge(self) -> MutEdge<'a, T, S, A> {
        MutEdge { graph: self.graph, id: self.id, }
    }

    pub fn expand<G>(mut self, state: T, g: G) -> MutEdge<'a, T, S, A> where G: FnOnce() -> S {
        match self.graph.state_ids.get_or_insert(state) {
            NamespaceInsertion::Present(target_id) => {
                if self.graph.path_exists(target_id, self.arc().source) {
                    self.arc_mut().target = Target::Cycle(target_id);
                } else {
                    self.arc_mut().target = Target::Expanded(target_id);
                }
            },
            NamespaceInsertion::New(target_id) => {
                self.arc_mut().target = Target::Expanded(target_id);
                self.graph.add_vertex(g()).parents.push(self.id);
            }
        }
        MutEdge { graph: self.graph, id: self.id, }
    }
}

pub struct EdgeAdder<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a mut Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> EdgeAdder<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    pub fn add<'s>(&'s mut self, data: A) -> MutEdge<'s, T, S, A> {
        let arc_id = ArcId(self.graph.arcs.len());
        self.graph.add_arc(data, self.id, Target::Unexpanded(()));
        MutEdge { graph: self.graph, id: arc_id, }
    }

    pub fn to_add(mut self, data: A) -> MutEdge<'a, T, S, A> {
        let arc_id = ArcId(self.graph.arcs.len());
        self.graph.add_arc(data, self.id, Target::Unexpanded(()));
        MutEdge { graph: self.graph, id: arc_id, }
    }
}
