#![feature(associated_type_defaults)]
#![feature(clone_from_slice)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

extern crate gtk;
extern crate cairo;

pub mod console_ui;
pub mod gtk_ui;

pub mod actions;
pub mod board;
// pub mod rules;

use std::cmp;
use std::fmt;
use std::slice;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Dwarf,
    Troll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    role: Role,
    name: String,
}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}

pub struct GameState {
    board: board::Cells,
    players: [Player; 2],
    first_player_active: bool,
    player_1_conceded: bool,
    player_2_conceded: bool,
}

impl GameState {
    pub fn new(board: board::Cells, player1_name: String, player2_name: String) -> Self {
        GameState {
            board: board,
            players: [Player { role: Role::Dwarf, name: player1_name, },
                      Player { role: Role::Troll, name: player2_name, },],
            first_player_active: true,
            player_1_conceded: false,
            player_2_conceded: false,
        }
    }
    
    pub fn new_default(player1_name: String, player2_name: String) -> Self {
        GameState::new(board::Cells::default(), player1_name, player2_name)
    }

    pub fn active_player(&self) -> &Player {
        if self.first_player_active {
            &self.players[0]
        } else {
            &self.players[1]
        }
    }

    pub fn role_actions<'s>(&'s self, r: Role) -> actions::ActionIterator<'s> {
        self.board.role_actions(r)
    }

    pub fn position_actions<'s>(&'s self, position: board::Coordinate) -> actions::ActionIterator<'s> {
        self.board.position_actions(position)
    }

    pub fn toggle_active_player(&mut self) {
        self.first_player_active = !self.first_player_active
    }

    pub fn do_action(&mut self, a: &actions::Action) {
        match a {
            &actions::Action::Concede =>
                if self.first_player_active {
                    self.player_1_conceded = true
                } else {
                    self.player_2_conceded = true
                },
            _ => self.board.do_action(a),
        }
        self.toggle_active_player();
    }

    pub fn terminated(&self) -> bool {
        self.player_1_conceded && self.player_2_conceded
    }
}
