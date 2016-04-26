use ::thud_game::Action;
use ::thud_game::coordinate::Coordinate;
use ::thud_ui_common::ThudState;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Passive {
    pub state: ThudState,
}

#[derive(Clone)]
pub struct Interactive {
    pub state: ThudState,
    pub mouse_down: Option<Coordinate>,
    pub mode: InputMode,
}

#[derive(Clone)]
pub enum InputMode {
    Inactive,
    Selected { from: Coordinate, actions: HashMap<Coordinate, Action>, },
    Targeted { from: Coordinate, to: Coordinate, action: Action,
               from_actions: HashMap<Coordinate, Action>, },

}

impl Interactive {
    pub fn new(state: ThudState) -> Self {
        Interactive { state: state,
                      mouse_down: None,
                      mode: InputMode::Inactive, }
    }

    pub fn is_inactive(&self) -> bool {
        match self {
            &InputMode::Inactive => true,
            _ => false,
        }
    }

    pub fn is_selected(&self) -> bool {
        match self {
            &InputMode::Selected { .. } => true,
            _ => false,
        }
    }

    pub fn is_targeted(&self) -> bool {
        match self {
            &InputMode::Targeted { from: _, to: _, action: _, from_actions: _, } => true,
            _ => false,
        }
    }
}
