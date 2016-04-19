use ::thud_game::Action;
use ::thud_game::board::{Coordinate};
use ::mcts::State;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Passive {
    pub state: State,
}

#[derive(Clone)]
pub struct Interactive {
    pub state: State,
    pub mouse_down: Option<Coordinate>,
    pub action: ActionState,
}

impl Interactive {
    pub fn new(state: State) -> Self {
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
