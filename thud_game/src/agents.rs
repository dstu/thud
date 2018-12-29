use crate::actions::{Action, ActionParseError};
use std::{error, fmt, result};
use std::io::{self, BufRead};
use std::str::FromStr;

/// Errors that may occur when querying an agent for its next move.
#[derive(Debug)]
pub enum Error {
  BadAction(ActionParseError),
  Exhausted,
  Wrapped(Box<(dyn error::Error + 'static)>),
}

impl From<ActionParseError> for Error {
  fn from(err: ActionParseError) -> Self {
    Error::BadAction(err)
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::Wrapped(Box::new(err))
  }
}

impl error::Error for Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Error::BadAction(e) => Some(&e),
      Error::Exhausted => None,
      Error::Wrapped(e) => Some(&*e),
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

type Result = result::Result<Action, Error>;

/// The interface that Thud-playing agents should implement.
pub trait Agent {
  /// Requests a move for the given game state. This move should be legal, and
  /// it is reasonable for an agent to assume that the move will be applied to
  /// `state` immediately afterwards.
  ///
  /// If an agent cannot propose a move (e.g., because it is in a bad internal
  /// state, or it has read the last move from a pre-recorded list), it should
  /// return an error.
  fn propose_move(&mut self, state: &crate::state::State) -> Result;
}

/// An agent that reads Thud moves from stdin. This agent does not lock stdin,
/// making it possible to have two differnt instances of it take turns reading
/// from stdin.
///
/// A prompt function may be specified, which will result in a prompt string
/// being printed before each move is read.
pub struct StdinAgent<F: Fn(&crate::state::State) -> String> {
  prompt: Option<F>,
}

impl<F: Fn(&crate::state::State) -> String> StdinAgent<F> {
  /// Returns an agent that will read moves from stdin without printing a prompt.
  pub fn new() -> Self {
    StdinAgent { prompt: None, }
  }

  /// Returns an agent that will evaluate `prompt` on the current game state and
  /// print the resulting string each time a move is requested.
  pub fn with_prompt(prompt: F) -> Self {
    StdinAgent { prompt: Some(prompt), }
  }
}

impl<F: Fn(&crate::state::State) -> String> Agent for StdinAgent<F> {
  fn propose_move(&mut self, state: &crate::state::State) -> Result {
    if let Some(prompt_fn) = self.prompt {
      print!("{}", prompt_fn(state));
    }
    let mut line = String::new();
    let bytes_read = io::stdin().lock().read_line(&mut line)?;
    if bytes_read == 0 {
      return Err(Error::Exhausted)
    }
    Ok(Action::from_str(line.trim())?)
  }
}

/// An agent that reads Thud moves from an input stream.
pub struct ReaderAgent<R: io::BufRead> {
  reader: R,
}

impl<R: io::BufRead> ReaderAgent<R> {
  pub fn new(reader: R) -> Self {
    ReaderAgent { reader, }
  }
}

impl<R: io::BufRead> Agent for ReaderAgent<R> {
  fn propose_move(&mut self, _state: &crate::state::State) -> Result {
    let mut line = String::new();
    let bytes_read = self.reader.read_line(&mut line)?;
    if bytes_read == 0 {
      return Err(Error::Exhausted)
    }
    Ok(Action::from_str(line.trim())?)
  }
}
