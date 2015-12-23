use ::actions;

use std::fmt;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::clone::Clone;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::RangeFrom;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ArcId(usize);

impl ArcId {
    fn as_usize(self) -> usize {
        let ArcId(value) = self;
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateId(usize);

impl StateId {
    fn as_usize(self) -> usize {
        let StateId(value) = self;
        value
    }
}

struct StateNamespace<T> where T: Hash + Eq + Clone {
    states: HashMap<T, StateId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum NamespaceInsertion {
    Present(StateId),
    New(StateId),
}

impl<T> StateNamespace<T> where T: Hash + Eq + Clone {
    fn new() -> Self {
        StateNamespace {
            states: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, state: T) -> NamespaceInsertion {
        let next_state_id = StateId(self.states.len());
        match self.states.entry(state) {
            Entry::Occupied(e) => NamespaceInsertion::Present(*e.get()),
            Entry::Vacant(e) => NamespaceInsertion::New(*e.insert(next_state_id)),
        }
    }

    fn get(&self, state: &T) -> Option<StateId> {
        self.states.get(state).map(|x| *x)
    }
}

pub enum Target<T, R> {
    Unexpanded(R),
    Cycle(T),
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

#[derive(Debug)]
struct Vertex<S> where S: Debug {
    data: S,
    parents: Vec<ArcId>,
    children: Vec<ArcId>,
}

#[derive(Debug)]
struct Arc<A> where A: Debug {
    data: A,
    source: StateId,
    target: Target<StateId, ()>,
}

pub struct Graph<T, S, A> where T: Hash + Eq + Clone, S: Debug, A: Debug {
    state_ids: StateNamespace<T>,
    vertices: Vec<Vertex<S>>,  // Indexed by StateId.
    arcs: Vec<Arc<A>>,  // Indexed by ArcId.
}

impl<T, S, A> Graph<T, S, A> where T: Hash + Eq + Clone, S: Debug, A: Debug {
    pub fn new() -> Self {
        Graph {
            state_ids: StateNamespace::new(),
            vertices: Vec::new(),
            arcs: Vec::new(),
        }
    }

    fn get_vertex(&self, state: StateId) -> &Vertex<S> {
        &self.vertices[state.as_usize()]
    }

    fn get_vertex_mut(&mut self, state: StateId) -> &mut Vertex<S> {
        &mut self.vertices[state.as_usize()]
    }

    fn get_arc(&self, arc: ArcId) -> &Arc<A> {
        &self.arcs[arc.as_usize()]
    }

    fn get_arc_mut(&mut self, arc: ArcId) -> &mut Arc<A> {
        &mut self.arcs[arc.as_usize()]
    }

    fn add_vertex(&mut self, data: S) -> &mut Vertex<S> {
        self.vertices.push(Vertex { data: data, parents: Vec::new(), children: Vec::new() });
        self.vertices.last_mut().unwrap()
    }

    fn add_arc(&mut self, data: A, source: StateId, target: Target<StateId, ()>) {
        let arc = Arc { data: data, source: source, target: target, };
        let arc_id = ArcId(self.arcs.len());
        if let Target::Expanded(target_id) = target {
            self.get_vertex_mut(target_id).parents.push(arc_id);
        }
        self.get_vertex_mut(source).children.push(arc_id);
        self.arcs.push(arc);
    }

    fn path_exists(&self, source: StateId, target: StateId) -> bool {
        let mut frontier = vec![target];
        while !frontier.is_empty() {
            let state = frontier.pop().unwrap();
            if source == state {
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

    pub fn get_node<'s>(&'s self, state: &T) -> Option<Node<'s, T, S, A>> {
        match self.state_ids.get(&state) {
            Some(id) => Some(Node { graph: self, id: id, }),
            None => None,
        }
    }

    pub fn get_node_mut<'s>(&'s mut self, state: &T) -> Option<MutNode<'s, T, S, A>> {
        match self.state_ids.get(state) {
            Some(id) => Some(MutNode { graph: self, id: id, }),
            None => None,
        }
    }
}

pub struct Node<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> Node<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn children(&self) -> &'a [ArcId] {
        &self.graph.get_vertex(self.id).children
    }

    fn child(&self, i: usize) -> ArcId {
        self.children()[i]
    }

    fn parents(&self) -> &'a [ArcId] {
        &self.graph.get_vertex(self.id).parents
    }

    pub fn data(&self) -> &'a S {
        &self.graph.get_vertex(self.id).data
    }

    pub fn is_leaf(&self) -> bool {
        self.children().is_empty()
    }

    pub fn is_root(&self) -> bool {
        self.parents().is_empty()
    }

    pub fn get_child_list(&self) -> ChildList<'a, T, S, A> {
        ChildList { graph: self.graph, id: self.id, }
    }

    pub fn get_parent_list(&self) -> ParentList<'a, T, S, A> {
        ParentList { graph: self.graph, id: self.id, }
    }
}

pub struct ChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> ChildList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }
    
    pub fn len(&self) -> usize {
        self.vertex().children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex().children.is_empty()
    }

    pub fn get_source_node(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.id, }
    }

    pub fn get_edge(&self, i: usize) -> Edge<'a, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().children[i], }
    }
}

pub struct ParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: StateId,
}

impl<'a, T, S, A> ParentList<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    pub fn target_node(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.id, }
    }

    pub fn get_edge(&self, i: usize) -> Edge<'a, T, S, A> {
        Edge { graph: self.graph, id: self.vertex().parents[i] }
    }
}

pub struct Edge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    graph: &'a Graph<T, S, A>,
    id: ArcId,
}

impl<'a, T, S, A> Edge<'a, T, S, A> where T: Hash + Eq + Clone + 'a, S: Debug + 'a, A: Debug + 'a {
    fn arc(&self) -> &'a Arc<A> {
        self.graph.get_arc(self.id)
    }

    pub fn get_data(&self) -> &'a A {
        &self.arc().data
    }

    pub fn get_source(&self) -> Node<'a, T, S, A> {
        Node { graph: self.graph, id: self.arc().source, }
    }

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

    pub fn data<'s>(&'s self) -> &'s S {
        &self.vertex().data
    }

    pub fn data_mut<'s>(&'s mut self) -> &'s mut S {
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

    pub fn data(&self) -> &A {
        &self.arc().data
    }

    pub fn data_mut(&mut self) -> &mut A {
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

    pub fn expand<F, G>(mut self, state: T, f: F, g: G) -> MutEdge<'a, T, S, A>
        where F: Fn(actions::Action) -> A, G: FnOnce() -> S {
            match self.graph.state_ids.get_or_insert(state) {
                NamespaceInsertion::Present(target_id) => {
                    if self.graph.path_exists(self.arc().source, target_id) {
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
    fn vertex_mut(&mut self) -> &mut Vertex<S> {
        self.graph.get_vertex_mut(self.id)
    }

    pub fn add<'s>(&'s mut self, data: A) -> MutEdge<'s, T, S, A> {
        let arc_id = ArcId(self.graph.arcs.len());
        self.vertex_mut().children.push(arc_id);
        self.graph.arcs.push(Arc { data: data, source: self.id, target: Target::Unexpanded(()), });
        MutEdge { graph: self.graph, id: arc_id, }
    }

    pub fn to_add(mut self, data: A) -> MutEdge<'a, T, S, A> {
        let arc_id = ArcId(self.graph.arcs.len());
        self.vertex_mut().children.push(arc_id);
        self.graph.arcs.push(Arc { data: data, source: self.id, target: Target::Unexpanded(()), });
        MutEdge { graph: self.graph, id: arc_id, }
    }
}
