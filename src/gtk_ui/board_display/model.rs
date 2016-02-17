use ::game;
use ::game::Action;
use ::game::board::{Coordinate};

use std::collections::HashMap;

#[derive(Clone)]
pub struct Passive {
    pub state: game::State,
}

#[derive(Clone)]
pub struct Interactive {
    pub state: game::State,
    pub mouse_down: Option<Coordinate>,
    pub action: ActionState,
}

impl Interactive {
    pub fn new(state: game::State) -> Self {
        Interactive { state: state,
                      mouse_down: None,
                      action: ActionState::Inactive, }
    }
}

#[derive(Clone)]
pub enum ActionState {
    Inactive,
    Selected { from: Coordinate, actions: HashMap<Coordinate, Action>, },
    Targeted { from: Coordinate, to: Coordinate, action: Action,
               from_actions: HashMap<Coordinate, Action>, },

}
