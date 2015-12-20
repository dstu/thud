use ::game;

use ::actions::Action;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::clone::Clone;
use std::default::Default;
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ActionTarget {
    Unexpanded,
    Cycle,
    State(StateId),
}

impl ActionTarget {
    fn unwrap(self) -> StateId {
        match self {
            ActionTarget::State(id) => id,
            _ => panic!("Can't unwrap {:?}", self),
        }
    }

    fn to_option(self) -> Option<StateId> {
        match self {
            ActionTarget::State(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Arc {
    action: Action,
    source: StateId,
    target: ActionTarget,
}

pub struct SearchGraph<S, A> where S: Default, A: Default {
    root_id: StateId,
    state_ids: StateNamespace,
    arc_ids: ArcNamespace,
    state_out: Vec<Vec<ArcId>>,  // Indexed by StateId.
    state_in: Vec<Vec<ArcId>>,  // Indexed by StateId.
    state_data: Vec<S>,  // Indexed by StateId.
    arcs: Vec<Arc>,  // Indexed by ArcId.
    arc_data: Vec<A>,  // Indexed by ArcId.
}

impl<S, A> SearchGraph<S, A> where S: Default, A: Default {
    pub fn new(root_state: &game::State) -> Self {
        let mut graph = SearchGraph {
            root_id: StateId(0),
            state_ids: StateNamespace::new(),
            arc_ids: ArcNamespace::new(),
            state_out: Vec::new(),
            state_in: Vec::new(),
            arcs: Vec::new(),
            state_data: Vec::new(),
            arc_data: Vec::new(),
        };
        graph.add_state(&root_state);
        graph
    }

    fn add_state(&mut self, state: &game::State) -> NamespaceInsertion {
        let actions = state.role_actions(state.active_player().role());
        match self.state_ids.get_or_insert(state.clone()) {
            NamespaceInsertion::New(state_id) => {
                self.state_data.push(Default::default());
                self.state_out.push(Vec::new());
                self.state_in.push(Vec::new());
                for action in actions {
                    self.add_arc(action, state_id, ActionTarget::Unexpanded);
                }
                assert!(self.state_data.len() == state_id.as_usize() + 1);
                assert!(self.state_out.len() == state_id.as_usize() + 1);
                assert!(self.state_in.len() == state_id.as_usize() + 1);
                NamespaceInsertion::New(state_id)
            },
            x @ _ => x,
        }
    }

    fn add_arc(&mut self, action: Action, source: StateId, target: ActionTarget) {
        let arc = Arc { action: action, source: source, target: target, };
        let arc_id = self.arc_ids.next_id();
        if let ActionTarget::State(target_id) = target {
            self.state_in[target_id.as_usize()].push(arc_id);
        }
        self.state_out[source.as_usize()].push(arc_id);
        self.arcs.push(arc);
        self.arc_data.push(Default::default());
        assert!(self.arcs.len() == arc_id.as_usize() + 1);
        assert!(self.arc_data.len() == arc_id.as_usize() + 1);
    }

    fn expand_action_target(&mut self, from_state: &game::State, id: ArcId) -> Option<StateId> {
        let (arc_source, arc_target) = {
            let arc = &self.arcs[id.as_usize()];
            (arc.source, arc.target)
        };
        let new_target = match arc_target {
            ActionTarget::Unexpanded => {
                match self.state_ids.states.get(from_state).map(|x| *x) {
                    Some(source_id) => {
                        if source_id != arc_source {
                            panic!("Source state ID {:?} does not match action source {:?}",
                                   source_id, arc_source);
                        }
                        let mut target_state = from_state.clone();
                        target_state.do_action(&self.arcs[id.as_usize()].action);
                        match self.add_state(&target_state) {
                            NamespaceInsertion::New(target_id) => {
                                self.state_in[target_id.as_usize()].push(id);
                                ActionTarget::State(target_id)
                            },
                            NamespaceInsertion::Present(target_id) => {
                                if self.path_exists(target_id, source_id) {
                                    ActionTarget::Cycle
                                } else {
                                    ActionTarget::State(target_id)
                                }
                            },
                        }
                    },
                    None =>
                        panic!("Source state supplied for action {:?} does not match known states",
                               self.arcs[id.as_usize()]),
                }
            },
            _ => panic!("Action {:?} already expanded", self.arcs[id.as_usize()]),
        };
        self.arcs[id.as_usize()].target = new_target;
    }

    fn path_exists(&self, source: StateId, target: StateId) -> bool {
        let mut frontier = vec![target];
        while !frontier.is_empty() {
            let state = frontier.pop().unwrap();
            if source == state {
                return true
            }
            for arc in self.state_out[state.as_usize()].iter().map(|&x| &self.arcs[x.as_usize()]) {
                if let ActionTarget::State(target_id) = arc.target {
                    frontier.push(target_id);
                }
            }
        }
        false
    }

    pub fn get_node<'s>(&'s self, state: &game::State) -> Option<Node<'s, S, A>> {
        match self.state_ids.get(state) {
            Some(id) => Some(Node { graph: self, state: state.clone(), id: id, }),
            None => None,
        }
    }

    pub fn get_node_mut<'s>(&'s mut self, state: &game::State) -> Option<MutNode<'s, S, A>> {
        match self.state_ids.get(state) {
            Some(id) => Some(MutNode { graph: self, state: state.clone(), id: id, }),
            None => None,
        }
    }

    pub fn promote_node_mut<'s>(&'s mut self, node: Node<'s, S, A>) -> MutNode<'s, S, A> {
        MutNode { graph: self, state: node.state, id: node.id, }
    }
}

pub struct Node<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a SearchGraph<S, A>,
    state: game::State,
    id: StateId,
}

impl<'a, S, A> Node<'a, S, A> where S: Default + 'a, A: Default + 'a {
    fn child_actions(&self) -> &'a [ArcId] {
        &self.graph.state_out[self.id.as_usize()]
    }

    fn parent_actions(&self) -> &'a [ArcId] {
        &self.graph.state_in[self.id.as_usize()]
    }

    pub fn data(&self) -> &'a S {
        &self.graph.state_data[self.id.as_usize()]
    }

    pub fn out_edges(&self) -> OutEdgeList<'a, S, A> {
        OutEdgeList { graph: self.graph, state: self.state, children: self.child_actions(), }
    }

    pub fn in_edges(&self) -> InEdgeList<'a, S, A> {
        InEdgeList { graph: self.graph, state: self.state, parents: self.parent_actions(), }
    }
}

pub enum OutEdge<'a, S, A> where S: Default + 'a, A: Default + 'a {
    Cycle,
    Unexpanded,
    Expanded(ExpandedData<'a, S, A>),
}

pub struct ExpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a SearchGraph<S, A>,
    state: game::State,
    id: ArcId,
}

impl<'a, S, A> ExpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    fn arc(&self) -> &'a Arc {
        &self.graph.arcs[self.id.as_usize()]
    }
    
    pub fn action(&self) -> &'a Action {
        &self.arc().action
    }

    pub fn source(&self) -> Node<'a, S, A> {
        Node { graph: self.graph, state: self.state, id: self.arc().source, }
    }

    pub fn target(&self) -> Node<'a, S, A> {
        let mut target_state = self.state.clone();
        target_state.do_action(self.action());
        Node { graph: self.graph, state: target_state, id: self.arc().target.unwrap(), }
    }

    pub fn data(&self) -> &'a A {
        &self.graph.arc_data[self.id.as_usize()]
    }
}

pub struct OutEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a SearchGraph<S, A>,
    state: game::State,
    children: &'a [ArcId],
}

impl<'a, S, A> OutEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn get(&self, i: usize) -> OutEdge<'a, S, A> {
        match self.graph.arcs[self.children[i].as_usize()].target {
            ActionTarget::Unexpanded => OutEdge::Unexpanded,
            ActionTarget::Cycle => OutEdge::Cycle,
            ActionTarget::State(_) =>
                OutEdge::Expanded(ExpandedData { graph: self.graph, state: self.state, id: self.children[i], }),
        }
    }
}

pub struct InEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a SearchGraph<S, A>,
    state: game::State,
    parents: &'a [ArcId],
}

impl<'a, S, A> InEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }

    pub fn len(&self) -> usize {
        self.parents.len()
    }

    pub fn get(&self, i: usize) -> ExpandedData<'a, S, A> {
        let mut parent_state = self.state.clone();
        ExpandedData { graph: self.graph, state: self.state, id: self.parents[i], }
    }
}

pub struct MutNode<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    state: game::State,
    id: StateId,
}

impl<'a, S, A> MutNode<'a, S, A> where S: Default + 'a, A: Default + 'a {
    fn child_actions(&self) -> &[ArcId] {
        &self.graph.state_out[self.id.as_usize()]
    }

    fn parent_actions(&self) -> &[ArcId] {
        &self.graph.state_in[self.id.as_usize()]
    }

    pub fn data(&self) -> &S {
        &self.graph.state_data[self.id.as_usize()]
    }

    pub fn data_mut(&mut self) -> &mut S {
        &mut self.graph.state_data[self.id.as_usize()]
    }

    pub fn out_edges<'s>(&'s self) -> OutEdgeList<'s, S, A> {
        OutEdgeList { graph: self.graph, state: self.state, children: self.child_actions(), }
    }

    pub fn out_edges_mut(mut self) -> MutOutEdgeList<'a, S, A> {
        MutOutEdgeList { graph: self.graph, state: self.state, source_id: self.id, }
    }

    pub fn in_edges<'s>(&'s self) -> InEdgeList<'s, S, A> {
        InEdgeList { graph: self.graph, parents: self.parent_actions(), }
    }

    pub fn in_edges_mut(mut self) -> MutInEdgeList<'a, S, A> {
        MutInEdgeList { graph: self.graph, target_id: self.id, }
    }
}

pub struct MutOutEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    state: game::State,
    source_id: StateId,
}

impl<'a, S, A> MutOutEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn is_empty(&self) -> bool {
        self.graph.state_out[self.source_id.as_usize()].is_empty()
    }

    pub fn len(&self) -> usize {
        self.graph.state_out[self.source_id.as_usize()].len()
    }

    pub fn get<'s>(&'s self, i: usize) -> OutEdge<'s, S, A> {
        match self.graph.arcs[self.graph.state_out[self.source_id.as_usize()][i].as_usize()].target {
            ActionTarget::Unexpanded => OutEdge::Unexpanded,
            ActionTarget::Cycle => OutEdge::Cycle,
            ActionTarget::State(_) =>
                OutEdge::Expanded(ExpandedData { graph: self.graph, id: self.graph.state_out[self.source_id.as_usize()][i], }),
        }
    }

    pub fn get_mut(self, i: usize) -> MutOutEdge<'a, S, A> {
        match self.graph.arcs[self.graph.state_out[self.source_id.as_usize()][i].as_usize()].target {
            ActionTarget::Unexpanded => {
                let id = self.graph.state_out[self.source_id.as_usize()][i];
                MutOutEdge::Unexpanded(MutUnexpandedData { graph: self.graph, state: self.state, id: id, })
            },
            ActionTarget::Cycle => MutOutEdge::Cycle,
            ActionTarget::State(_) => {
                let id = self.graph.state_out[self.source_id.as_usize()][i];
                MutOutEdge::Expanded(MutExpandedData { graph: self.graph, state: self.state, id: id, })
            },
        }
    }
}

pub enum MutOutEdge<'a, S, A> where S: Default + 'a, A: Default + 'a {
    Cycle,
    Unexpanded(MutUnexpandedData<'a, S, A>),
    Expanded(MutExpandedData<'a, S, A>),
}

pub struct MutUnexpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    state: game::State,
    id: ArcId,
}

impl<'a, S, A> MutUnexpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn action(&self) -> &Action {
        &self.graph.arcs[self.id.as_usize()].action
    }

    pub fn source<'s>(&'s self) -> Node<'s, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].source;
        Node { graph: self.graph, id: id, }
    }

    pub fn source_mut(self) -> MutNode<'a, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].source;
        MutNode { graph: self.graph, id: id, }
    }

    pub fn expand(mut self) -> Option<MutExpandedData<'a, S, A>> {
        self.graph.expand_action_target(&self.state, self.id).map(|target_id| {
            MutExpandedData { graph: self.graph, id: target_id, }
        })
    }
}

pub struct MutExpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    state: game::State,
    id: ArcId,
}

impl<'a, S, A> MutExpandedData<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn action(&self) -> &Action {
        &self.graph.arcs[self.id.as_usize()].action
    }

    pub fn source<'s>(&'s self) -> Node<'s, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].source;
        Node { graph: self.graph, id: id, }
    }

    pub fn source_mut(self) -> MutNode<'a, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].source;
        MutNode { graph: self.graph, id: id, }
    }

    pub fn target<'s>(&'s self) -> Node<'s, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].target.unwrap();
        Node { graph: self.graph, id: id, }
    }

    pub fn target_mut(self) -> MutNode<'a, S, A> {
        let id = self.graph.arcs[self.id.as_usize()].target.unwrap();
        MutNode { graph: self.graph, id: id, }
    }

    pub fn data(&self) -> &A {
        &self.graph.arc_data[self.id.as_usize()]
    }

    pub fn data_mut(&mut self) -> &mut A {
        &mut self.graph.arc_data[self.id.as_usize()]
    }
}

pub struct MutInEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    graph: &'a mut SearchGraph<S, A>,
    target_id: StateId,
}

impl<'a, S, A> MutInEdgeList<'a, S, A> where S: Default + 'a, A: Default + 'a {
    pub fn is_empty(&self) -> bool {
        self.graph.state_in[self.target_id.as_usize()].is_empty()
    }

    pub fn len(&self) -> usize {
        self.graph.state_in[self.target_id.as_usize()].len()
    }

    pub fn get<'s>(&'s self, i: usize) -> ExpandedData<'s, S, A> {
        ExpandedData { graph: self.graph, id: self.graph.state_in[self.target_id.as_usize()][i], }
    }

    pub fn get_mut(self, i: usize) -> MutExpandedData<'a, S, A> {
        let id = self.graph.state_in[self.target_id.as_usize()][i];
        MutExpandedData { graph: self.graph, id: id, }
    }
}
