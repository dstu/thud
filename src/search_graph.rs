use ::board;
use ::game;

use ::actions::Action;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::clone::Clone;
use std::default::Default;
use std::ops::RangeFrom;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActionId(usize);

impl ActionId {
    fn as_usize(self) -> usize {
        let ActionId(value) = self;
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
    state_id_generator: RangeFrom<usize>,
    states: HashMap<game::State, StateId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum NamespaceInsertion {
    Present(StateId),
    New(StateId),
}

impl NamespaceInsertion {
    fn unwrap(self) -> StateId {
        match self {
            NamespaceInsertion::Present(s) => s,
            NamespaceInsertion::New(s) => s,
        }
    }
}

impl StateNamespace {
    fn new() -> Self {
        StateNamespace {
            state_id_generator: 0..,
            states: HashMap::new(),
        }
    }

    fn next_id(&mut self) -> StateId {
        match self.state_id_generator.next() {
            Some(id) => StateId(id),
            None => panic!("Exhausted state ID namespace"),
        }
    }

    fn get_or_insert(&mut self, state: game::State) -> NamespaceInsertion {
        match self.states.entry(state) {
            Entry::Occupied(e) => NamespaceInsertion::Present(*e.get()),
            Entry::Vacant(e) => NamespaceInsertion::New(*e.insert(self.next_id())),
        }
    }

    fn get(&self, state: &game::State) -> Option<StateId> {
        self.states.get(state).map(|x| *x)
    }
}

struct ActionNamespace {
    action_id_generator: RangeFrom<usize>,
}

impl ActionNamespace {
    fn new() -> Self {
        ActionNamespace {
            action_id_generator: 0..,
        }
    }

    fn next_id(&mut self) -> ActionId {
        match self.action_id_generator.next() {
            Some(id) => ActionId(id),
            None => panic!("Exhausted action ID namespace"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ActionTarget {
    Unexpanded,
    Cycle,
    State(StateId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ActionArc {
    action: Action,
    source: StateId,
    target: ActionTarget,
}

pub struct SearchGraph<S, A> where S: Default, A: Default {
    root_id: StateId,
    state_ids: StateNamespace,
    action_ids: ActionNamespace,
    // Also consider making edges into an intrusive linked list, with head an
    // Option<EdgeNode> and EdgeNode<A> { data: A, action: Action, target:
    // StateId, next: Option<EdgeNode> }.
    state_out_edges: Vec<Vec<ActionId>>,  // Indexed by StateId.
    state_in_edges: Vec<Vec<ActionId>>,  // Indexed by StateId.
    action_arcs: Vec<ActionArc>,  // Indexed by ActionId.
    state_data: Vec<S>,  // Indexed by StateId.
    action_data: Vec<A>,  // Indexed by ActionId.
}

impl<S, A> SearchGraph<S, A> where S: Default, A: Default {
    pub fn new(root_state: game::State) -> Self {
        let mut graph = SearchGraph {
            root_id: StateId(0),
            state_ids: StateNamespace::new(),
            action_ids: ActionNamespace::new(),
            state_out_edges: Vec::new(),
            state_in_edges: Vec::new(),
            action_arcs: Vec::new(),
            state_data: Vec::new(),
            action_data: Vec::new(),
        };
        graph.add_state(root_state);
        graph
    }

    fn add_state(&mut self, state: game::State) -> NamespaceInsertion {
        match self.state_ids.get_or_insert(state) {
            (state, NamespaceInsertion::New(state_id)) => {
                self.state_data.push(Default::default());
                self.state_out_edges.push(Vec::new());
                self.state_in_edges.push(Vec::new());
                for action in state.role_actions(state.active_player().role()) {
                    self.add_action_arc(action, state_id, ActionTarget::Unexpanded);
                }
                assert!(self.state_data.len() == state_id.as_usize() + 1);
                assert!(self.state_out_edges.len() == state_id.as_usize() + 1);
                assert!(self.state_in_edges.len() == state_id.as_usize() + 1);
                NamespaceInsertion::New(state_id)
            },
            (_, insertion) => insertion,
        }
    }

    fn add_action_arc(&mut self, action: Action, source: StateId, target: ActionTarget) {
        let arc = ActionArc { action: action, source: source, target: target, };
        let action_id = self.action_ids.next_id();
        self.state_out_edges[source.as_usize()].push(action_id);
        self.action_arcs.push(arc);
        self.action_data.push(Default::default());
        assert!(self.action_arcs.len() == action_id.as_usize() + 1);
        assert!(self.action_data.len() == action_id.as_usize() + 1);
    }

    fn expand_action_target(&mut self, from_state: &game::State, id: ActionId) {
        let arc: &mut ActionArc = &mut self.action_arcs[id.as_usize()];
        match arc.target {
            ActionTarget::Unexpanded => {
                match self.state_ids.states.get(from_state) {
                    Some(source_id) => {
                        if *source_id != arc.source {
                            panic!("Source state ID {:?} does not match action source {:?}",
                                   source_id, arc.source);
                        }
                        let mut target_state = from_state.clone();
                        target_state.do_action(&arc.action);
                        match self.add_state(target_state) {
                            NamespaceInsertion::New(target_id) =>
                                self.state_in_edges[target_id.as_usize()].push(id),
                            NamespaceInsertion::Present(target_id) => {
                                if self.path_exists(target_id, *source_id) {
                                    arc.target = ActionTarget::Cycle;
                                } else {
                                    arc.target = ActionTarget::State(target_id);
                                }
                            },
                        }
                    },
                    None =>
                        panic!("Source state supplied for action {:?} does not match known states",
                               arc),
                }
            },
            _ => panic!("Action {:?} already expanded", arc),
        }
    }

    fn path_exists(self, source: StateId, target: StateId) -> bool {
        let mut frontier = vec![target];
        while !frontier.is_empty() {
            let state = frontier.pop().unwrap();
            if source == state {
                return true
            }
            for arc in self.state_out_edges[state.as_usize()].iter().map(|&x| &self.action_arcs[x.as_usize()]) {
                if let ActionTarget::State(target_id) = arc.target {
                    frontier.push(target_id);
                }
            }
        }
        false
    }

    // pub fn get_state<'s>(&'s mut self, states: game::State) -> StateNode<'s, S, A> {
        
    // }
}

pub struct StateNode<'a, S: 'a, A: 'a> where S: Default, A: Default {
    graph: &'a mut SearchGraph<S, A>,
    id: StateId,
}

impl<'a, S: 'a, A: 'a> StateNode<'a, S, A> where S: Default, A: Default {
    fn child_actions(&self) -> &[ActionId] {
        &self.graph.state_out_edges[self.id.as_usize()]
    }

    fn parent_actions(&self) -> &[ActionId] {
        &self.graph.state_in_edges[self.id.as_usize()]
    }

    pub fn data(&self) -> &S {
        &self.graph.state_data[self.id.as_usize()]
    }
}
