#![feature(associated_type_defaults)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

extern crate cairo;
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate log;
extern crate rand;
extern crate search_graph;

pub mod game;

pub mod mcts;

pub mod console_ui;
pub mod gtk_ui;
