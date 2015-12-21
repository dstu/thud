use ::game;

use std::fmt;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::clone::Clone;
use std::default::Default;
use std::fmt::Debug;
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

struct StateNamespace {
    states: HashMap<game::State, StateId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum NamespaceInsertion {
    Present(StateId),
    New(StateId),
}

impl StateNamespace {
    fn new() -> Self {
        StateNamespace {
            states: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, state: game::State) -> NamespaceInsertion {
        let next_state_id = StateId(self.states.len());
        match self.states.entry(state) {
            Entry::Occupied(e) => NamespaceInsertion::Present(*e.get()),
            Entry::Vacant(e) => NamespaceInsertion::New(*e.insert(next_state_id)),
        }
    }

    fn get(&self, state: &game::State) -> Option<StateId> {
        self.states.get(state).map(|x| *x)
    }
}

struct ArcNamespace {
    arc_id_generator: RangeFrom<usize>,
}

impl ArcNamespace {
    fn new() -> Self {
        ArcNamespace {
            arc_id_generator: 0..,
        }
    }

    fn next_id(&mut self) -> ArcId {
        match self.arc_id_generator.next() {
            Some(id) => ArcId(id),
            None => panic!("Exhausted arc ID namespace"),
        }
    }
}

pub enum Target<T> {
    Unexpanded,
    Cycle(T),
    Expanded(T),
}

impl<T> Target<T> {
    fn to_option(self) -> Option<T> {
        match self {
            Target::Expanded(t) => Some(t),
            _ => None,
        }
    }
}

impl<T> Clone for Target<T> where T: Clone {
    fn clone(&self) -> Self {
        match self {
            &Target::Cycle(ref t) => Target::Cycle(t.clone()),
            &Target::Unexpanded => Target::Unexpanded,
            &Target::Expanded(ref t) => Target::Expanded(t.clone()),
        }
    }
}

impl<T> Copy for Target<T> where T: Copy { }

impl<T> fmt::Debug for Target<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Target::Cycle(ref t) => write!(f, "Cycle({:?})", t),
            Target::Unexpanded => write!(f, "Unexpanded"),
            Target::Expanded(ref t) => write!(f, "Expanded({:?})", t),
        }
    }
}

#[derive(Debug)]
struct Vertex<S> where S: Default + Debug {
    data: S,
    parents: Vec<ArcId>,
    children: Vec<ArcId>,
}

impl<S> Default for Vertex<S> where S: Debug + Default  {
    fn default() -> Self {
        Vertex { data: Default::default(),
                 parents: Vec::new(),
                 children: Vec::new(), }
    }
}

#[derive(Debug)]
struct Arc<A> where A: Debug + Default {
    data: A,
    source: StateId,
    target: Target<StateId>,
}

pub struct SearchGraph<S, A> where S: Debug + Default, A: Debug + Default {
    state_ids: StateNamespace,
    arc_ids: ArcNamespace,
    vertices: Vec<Vertex<S>>,  // Indexed by StateId.
    arcs: Vec<Arc<A>>,  // Indexed by ArcId.
}

impl<S, A> SearchGraph<S, A> where S: Debug + Default, A: Debug + Default {
    pub fn new() -> Self {
        SearchGraph {
            state_ids: StateNamespace::new(),
            arc_ids: ArcNamespace::new(),
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

    // fn get_or_add_state_ref(&mut self, state: &game::State) -> NamespaceInsertion {
    //     let key = state.clone();
    //     match self.state_ids.get_or_insert(key) {
    //         NamespaceInsertion::New(state_id) => {
    //             self.vertices.push(Default::default());
    //             for action in state.role_actions(state.active_player().role()) {
    //                 self.add_arc(action, state_id, Target::Unexpanded);
    //             }
    //             assert!(self.vertices.len() == state_id.as_usize() + 1);
    //             NamespaceInsertion::New(state_id)
    //         },
    //         x @ _ => x,
    //     }
    // }

    // fn get_or_add_state(&mut self, state: game::State) -> NamespaceInsertion {
    //     let key = state.clone();
    //     match self.state_ids.get_or_insert(key) {
    //         NamespaceInsertion::New(state_id) => {
    //             self.vertices.push(Default::default());
    //             for action in state.role_actions(state.active_player().role()) {
    //                 self.add_arc(action, state_id, Target::Unexpanded);
    //             }
    //             assert!(self.vertices.len() == state_id.as_usize() + 1);
    //             NamespaceInsertion::New(state_id)
    //         },
    //         x @ _ => x,
    //     }        
    // }

    fn get_arc(&self, arc: ArcId) -> &Arc<A> {
        &self.arcs[arc.as_usize()]
    }

    fn get_arc_mut(&mut self, arc: ArcId) -> &mut Arc<A> {
        &mut self.arcs[arc.as_usize()]
    }

    fn add_arc(&mut self, data: A, source: StateId, target: Target<StateId>) {
        let arc = Arc { data: data, source: source, target: target, };
        let arc_id = self.arc_ids.next_id();
        if let Target::Expanded(target_id) = target {
            self.get_vertex_mut(target_id).parents.push(arc_id);
        }
        self.get_vertex_mut(source).children.push(arc_id);
        self.arcs.push(arc);
        assert!(self.arcs.len() == arc_id.as_usize() + 1);
    }

    // fn expand_target(&mut self, from_state: &game::State, id: ArcId) -> Target<StateId> {
    //     let (arc_source, arc_target) = {
    //         let arc = &self.arcs[id.as_usize()];
    //         (arc.source, arc.target)
    //     };
    //     if let Target::Unexpanded = arc_target {
    //         let mut target_state = from_state.clone();
    //         target_state.do_action(&self.get_arc(id).action);
    //         match self.get_or_add_state(target_state) {
    //             NamespaceInsertion::New(target_id) => {
    //                 self.vertices.push(Default::default());
    //                 self.get_arc_mut(id).target = Target::Expanded(target_id);
    //             },
    //             NamespaceInsertion::Present(target_id) => {
    //                 if self.path_exists(target_id, arc_source) {
    //                     self.get_arc_mut(id).target = Target::Cycle(target_id);
    //                 } else {
    //                     self.get_arc_mut(id).target = Target::Expanded(target_id);
    //                 }
    //             },
    //         }
    //         self.get_arc(id).target
    //     } else {
    //         panic!("Arc {:?} already expanded", self.get_arc(id));
    //     }
    // }

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

    pub fn get_node<'s>(&'s self, state: game::State) -> Option<Node<'s, S, A>> {
        match self.state_ids.get(&state) {
            Some(id) => Some(Node { graph: self, id: id, }),
            None => None,
        }
    }

    // pub fn get_node_mut<'s>(&'s mut self, state: game::State) -> Option<MutNode<'s, S, A>> {
    //     match self.state_ids.get(&state) {
    //         Some(id) => Some(MutNode { graph: self, state: state, id: id, }),
    //         None => None,
    //     }
    // }

    // pub fn promote_node_mut<'s>(&'s mut self, node: Node<'s, S, A>) -> MutNode<'s, S, A> {
    //     MutNode { graph: self, id: node.id, }
    // }
}

pub struct Node<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> Node<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
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

    pub fn child_list(&self) -> ChildList<'a, S, A> {
        ChildList { graph: self.graph, id: self.id, }
    }

    pub fn parent_list(&self) -> ParentList<'a, S, A> {
        ParentList { graph: self.graph, id: self.id, }
    }
}

pub struct ChildList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> ChildList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }
    
    pub fn len(&self) -> usize {
        self.vertex().children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex().children.is_empty()
    }

    pub fn get_edge(&self, i: usize) -> Edge<'a, S, A> {
        Edge { graph: self.graph, id: self.vertex().children[i], }
    }
}

pub struct ParentList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> ParentList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn vertex(&self) -> &'a Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    pub fn get_edge(&self, i: usize) -> Edge<'a, S, A> {
        Edge { graph: self.graph, id: self.vertex().parents[i] }
    }
}

pub struct Edge<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a SearchGraph<S, A>,
    id: ArcId,
}

impl<'a, S, A> Edge<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn arc(&self) -> &'a Arc<A> {
        self.graph.get_arc(self.id)
    }

    pub fn get_data(&self) -> &'a A {
        &self.arc().data
    }

    pub fn get_source(&self) -> Node<'a, S, A> {
        Node { graph: self.graph, id: self.arc().source, }
    }

    pub fn get_target(&self) -> Target<Node<'a, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(Node { graph: self.graph, id: id, }),
            Target::Unexpanded => Target::Unexpanded,
            Target::Expanded(id) => Target::Expanded(Node { graph: self.graph, id: id, }),
        }
    }
}

pub struct MutNode<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> MutNode<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
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

    pub fn child_list<'s>(&'s self) -> ChildList<'s, S, A> {
        ChildList { graph: self.graph, id: self.id, }
    }

    pub fn child_list_mut<'s>(&'s mut self) -> MutChildList<'s, S, A> {
        MutChildList { graph: self.graph, id: self.id, }
    }

    pub fn to_child_list_mut(self) -> MutChildList<'a, S, A> {
        MutChildList { graph: self.graph, id: self.id, }
    }

    pub fn parent_list<'s>(&'s self) -> ParentList<'s, S, A> {
        ParentList { graph: self.graph, id: self.id, }
    }

    pub fn parent_list_mut<'s>(&'s mut self) -> MutParentList<'s, S, A> {
        MutParentList { graph: self.graph, id: self.id, }
    }

    pub fn to_parent_list_mut(self) -> MutParentList<'a, S, A> {
        MutParentList { graph: self.graph, id: self.id, }
    }
}

pub struct MutChildList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> MutChildList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn vertex<'s>(&'s self) -> &'s Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().children.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertex().children.is_empty()
    }

    pub fn get_edge<'s>(&'s self, i: usize) -> Edge<'s, S, A> {
        Edge { graph: self.graph, id: self.vertex().children[i], }
    }

    pub fn get_edge_mut<'s>(&'s mut self, i: usize) -> MutEdge<'s, S, A> {
        let id = self.vertex().children[i];
        MutEdge { graph: self.graph, id: id, }
    }

    pub fn to_edge_mut(self, i: usize) -> MutEdge<'a, S, A> {
        let id = self.vertex().children[i];
        MutEdge { graph: self.graph, id: id, }
    }
}

pub struct MutParentList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S, A> MutParentList<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn vertex<'s>(&'s self) -> &'s Vertex<S> {
        self.graph.get_vertex(self.id)
    }

    pub fn len(&self) -> usize {
        self.vertex().parents.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vertex().parents.is_empty()
    }

    pub fn get_edge<'s>(&'s self, i: usize) -> Edge<'s, S, A> {
        Edge { graph: self.graph, id: self.vertex().parents[i], }
    }

    pub fn get_edge_mut<'s>(&'s mut self, i: usize) -> MutEdge<'s, S, A> {
        let id = self.vertex().parents[i];
        MutEdge { graph: self.graph, id: id, }
    }

    pub fn to_edge_mut(self, i: usize) -> MutEdge<'a, S, A> {
        let id = self.vertex().parents[i];
        MutEdge { graph: self.graph, id: id, }
    }
}

pub struct MutEdge<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    id: ArcId,
}

impl<'a, S, A> MutEdge<'a, S, A> where S: Debug + Default + 'a, A: Debug + Default + 'a {
    fn arc(&self) -> &Arc<A> {
        self.graph.get_arc(self.id)
    }

    fn arc_mut(&mut self) -> &mut Arc<A> {
        self.graph.get_arc_mut(self.id)
    }

    pub fn get_data(&self) -> &A {
        &self.arc().data
    }

    pub fn get_data_mut(&mut self) -> &mut A {
        &mut self.arc_mut().data
    }

    pub fn get_target<'s>(&'s self) -> Target<Node<'s, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(Node { graph: self.graph, id: id, }),
            Target::Unexpanded => Target::Unexpanded,
            Target::Expanded(id) =>
                Target::Expanded(Node { graph: self.graph, id: id, }),
        }
    }

    pub fn get_target_mut<'s>(&'s mut self) -> Target<MutNode<'s, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(MutNode { graph: self.graph, id: id, }),
            Target::Unexpanded => Target::Unexpanded,
            Target::Expanded(id) =>
                Target::Expanded(MutNode { graph: self.graph, id: id, }),
        }
    }

    pub fn to_target(self) -> Target<MutNode<'a, S, A>> {
        match self.arc().target {
            Target::Cycle(id) => Target::Cycle(MutNode { graph: self.graph, id: id, }),
            Target::Unexpanded => Target::Unexpanded,
            Target::Expanded(id) =>
                Target::Expanded(MutNode { graph: self.graph, id: id, }),
        }
    }
}
