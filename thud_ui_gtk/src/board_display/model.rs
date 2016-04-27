use ::thud_game::{Action, Role};
use ::thud_game::coordinate::Coordinate;
use ::thud_ui_common::ThudState;

use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum InteractiveRoles {
    One(Role),
    Both,
}

impl InteractiveRoles {
    pub fn is_interactive(&self, r: &Role) -> bool {
        match *self {
            InteractiveRoles::One(active) => active == *r,
            InteractiveRoles::Both => true,
        }
    }
}

#[derive(Clone)]
pub struct Interactive {
    /// Game state.
    pub state: ThudState,
    /// Most recent board coordinate where user pressed mouse button.
    pub mouse_down: Option<Coordinate>,
    /// State of user input.
    pub input_mode: InputMode,
    /// Roles that may interact when it's their turn.
    pub interactive_roles: InteractiveRoles,
}

#[derive(Clone)]
pub enum InputMode {
    /// Input disabled (e.g., not player's turn, currently processing input).
    Inactive,
    /// Input active but empty.
    Waiting,
    /// Player has selected piece at `from` with available actions `actions`.
    Selected { from: Coordinate, actions: HashMap<Coordinate, Action>, },
    /// Player is targeting a move from `from` to `to`, with other
    /// `from_actions` the other actions for the piece at `from`.
    Targeted { from: Coordinate, to: Coordinate, action: Action,
               from_actions: HashMap<Coordinate, Action>, },

}

impl Interactive {
    pub fn new(state: ::thud_ui_common::ThudState, interactive_roles: InteractiveRoles) -> Self {
        let initial_input_mode =
            if interactive_roles.is_interactive(state.active_role()) {
                InputMode::Waiting
            } else {
                InputMode::Inactive
            };
        Interactive {
            state: state,
            mouse_down: None,
            input_mode: initial_input_mode,
            interactive_roles: interactive_roles,
        }
    }
}

impl InputMode {
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
