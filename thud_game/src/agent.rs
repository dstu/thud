use crate::actions::Action;
use std::fs::File;
use std::future::Future;
use std::io::{self, BufRead};
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread::{self, JoinHandle};
use std::{error, fmt, result};

pub type Result = result::Result<Action, Box<dyn error::Error + Send>>;

/// Error states unique to [Agent](trait.Agent.html)s that read actions in from a text stream.
#[derive(Debug)]
pub enum ReaderAgentError {
  BadMove(String),
  Exhausted,
  Io(io::Error),
}

impl error::Error for ReaderAgentError {
  fn cause(&self) -> Option<&dyn error::Error> {
    match self {
      ReaderAgentError::BadMove(_) | ReaderAgentError::Exhausted => None,
      ReaderAgentError::Io(ref e) => Some(e),
    }
  }
}

impl fmt::Display for ReaderAgentError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ReaderAgentError::BadMove(s) => write!(f, "bad move string '{}'", s),
      ReaderAgentError::Exhausted => write!(f, "exhausted list of moves"),
      ReaderAgentError::Io(e) => write!(f, "{}", e),
    }
  }
}

impl From<io::Error> for ReaderAgentError {
  fn from(e: io::Error) -> Self {
    ReaderAgentError::Io(e)
  }
}

/// Implemented for agents that can play Thud.
pub trait Agent: Send {
  fn propose_action(&mut self, state: &crate::state::State) -> Result;
}

/// [Agent](trait.Agent.html) that reads moves from stdin.
///
/// This agent only locks stdin when it is reading a move, so multiple instances
/// of it may be used at once.
pub struct StdinReaderAgent {}

unsafe impl Send for StdinReaderAgent {}

impl Agent for StdinReaderAgent {
  /// Tries to interpret the next line available from `stdin` as a game action.
  fn propose_action(&mut self, _state: &crate::state::State) -> Result {
    let mut line = String::new();
    let bytes_read = match io::stdin().lock().read_line(&mut line) {
      Ok(n) => n,
      Err(e) => return Err(Box::new(ReaderAgentError::Io(e))),
    };
    if bytes_read == 0 {
      Err(Box::new(ReaderAgentError::Exhausted))
    } else {
      match Action::from_str(line.trim()) {
        Ok(a) => Ok(a),
        Err(_) => Err(Box::new(ReaderAgentError::BadMove(line))),
      }
    }
  }
}

/// [Agent](trait.Agent.html) that reads moves from lines of a `BufRead`.
pub struct BufReaderAgent<R: BufRead> {
  reader: R,
}

impl<R: BufRead> BufReaderAgent<R> {
  pub fn new(reader: R) -> Self {
    BufReaderAgent { reader }
  }
}

impl BufReaderAgent<io::BufReader<File>> {
  pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    let file = File::open(path)?;
    Ok(BufReaderAgent {
      reader: io::BufReader::new(file),
    })
  }
}

unsafe impl<R: BufRead> Send for BufReaderAgent<R> {}

impl<R: BufRead> Agent for BufReaderAgent<R> {
  /// Tries to interpret the next line of text in the underlying reader as a game
  /// action.
  fn propose_action(&mut self, _state: &crate::state::State) -> Result {
    let mut line = String::new();
    let bytes_read = match self.reader.read_line(&mut line) {
      Ok(n) => n,
      Err(e) => return Err(Box::new(ReaderAgentError::Io(e))),
    };
    if bytes_read == 0 {
      Err(Box::new(ReaderAgentError::Exhausted))
    } else {
      match Action::from_str(line.trim()) {
        Ok(a) => Ok(a),
        Err(_) => Err(Box::new(ReaderAgentError::BadMove(line))),
      }
    }
  }
}

/// Wraps around an asynchronous thread that executes an agent's move processing.
struct AgentFuture {
  rx: Receiver<Result>,
  _thread_handle: JoinHandle<()>,
}

impl AgentFuture {
  fn new(agent: Arc<Mutex<dyn Agent>>, state: &crate::state::State) -> Self {
    let (tx, rx) = channel();
    let state = state.clone();
    let thread_handle = thread::spawn(move || {
      let result = match agent.lock() {
        Ok(mut agent) => agent.propose_action(&state),
        Err(_) => panic!("agent mutex poisoned"),
      };
      tx.send(result)
        .expect("failed to send proposed action from agent thread")
    });
    AgentFuture {
      rx,
      _thread_handle: thread_handle,
    }
  }
}

impl Future for AgentFuture {
  type Output = Result;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result> {
    match self.rx.try_recv() {
      Ok(r) => Poll::Ready(r),
      Err(TryRecvError::Empty) => Poll::Pending,
      Err(e) => Poll::Ready(Err(Box::new(e))),
    }
  }
}

/// Returns a future that will query an [Agent](trait.Agent.html) off-thread and yield the result of
/// calling its `propose_action` method.
pub fn query_agent(
  agent: Arc<Mutex<dyn Agent>>,
  state: &crate::state::State,
) -> impl Future<Output = Result> {
  AgentFuture::new(agent, state)
}
