#![feature(associated_type_defaults)]
#![feature(const_fn)]
#![feature(custom_attribute)]
#![feature(fn_traits)]
#![feature(reflect_marker)]
#![feature(unboxed_closures)]
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]

#[cfg(feature = "ai-mcts")]
extern crate mcts;
#[cfg(test)]
extern crate quickcheck;
#[cfg(feature = "ai-mcts")]
extern crate syncbox;

#[macro_use]
extern crate log;
#[macro_use]
pub mod coordinate;
pub mod board;
#[macro_use]
pub mod actions;
#[cfg(any(feature = "ai", feature = "ai-mcts"))]
pub mod ai;
pub mod end;
pub mod state;
pub mod util;

pub use actions::Action;

use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Role {
  Dwarf,
  Troll,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UnrecognizedRoleError(String);

impl Role {
  pub fn index(self) -> usize {
    match self {
      Role::Dwarf => 0,
      Role::Troll => 1,
    }
  }

  pub fn toggle(self) -> Self {
    match self {
      Role::Dwarf => Role::Troll,
      Role::Troll => Role::Dwarf,
    }
  }
}

impl FromStr for Role {
  type Err = UnrecognizedRoleError;

  fn from_str(s: &str) -> Result<Self, UnrecognizedRoleError> {
    match s.to_lowercase().as_str() {
      "dwarf" => Ok(Role::Dwarf),
      "troll" => Ok(Role::Troll),
      _ => Err(UnrecognizedRoleError(s.to_string())),
    }
  }
}

impl fmt::Display for UnrecognizedRoleError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let UnrecognizedRoleError(ref s) = *self;
    write!(f, "Unrecognized role: {}", s)
  }
}

impl Error for UnrecognizedRoleError {
  fn description(&self) -> &str {
    "Unrecognized role"
  }
}
