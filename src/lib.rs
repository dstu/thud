#![feature(associated_type_defaults)]
#![feature(clone_from_slice)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

extern crate gtk;
extern crate cairo;
extern crate rand;

pub mod actions;
pub mod board;
pub mod game;

pub mod mcts;
pub mod search_graph;

pub mod console_ui;
pub mod gtk_ui;
