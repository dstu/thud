use clap::{App, Arg, ArgMatches};
use std::collections::HashMap;
use std::{error, fmt, result};
use thud_game::agent::{self, Agent};

pub const FLAG_PLAYER_1_AGENT: &'static str = "player_1_agent";
pub const FLAG_PLAYER_2_AGENT: &'static str = "player_1_agent";

#[derive(Debug)]
pub enum Error {
  /// No agent name was given.
  NoAgentIdentified,
  /// The named agent is unavailable.
  InvalidAgent(String),
  /// The the agent named `agent` could not be created because of an error with
  /// the parameter named `parameter`.
  InvalidAgentParameter { agent: String, parameter: String, error: Option<Box<dyn error::Error>>, },
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

type Result = result::Result<Box<dyn Agent>, Error>;

pub trait AgentBuilder {
  /// Returns the name of the agent.
  fn name(&self) -> &str;

  /// Registers arguments for configuring this agent on `app`.
  fn register_args<'a, 'b>(&'a self, app: App<'a, 'b> ) -> App<'a, 'b> where 'a: 'b;

  /// Creates an instance of this agent from `matches`, which should be the
  /// results of reading a configuration.
  fn build(&self, matches: &ArgMatches) -> result::Result<Box<dyn Agent>, Error>;
}

/// A collection of `AgentBuilder`s that can instantiate `Agent`s by name.
pub struct AgentRegistry {
  agents: HashMap<String, Box<dyn AgentBuilder>>,
}

impl AgentRegistry {
  /// Creates an empty registry.
  pub fn new() -> Self {
    AgentRegistry { agents: HashMap::new(), }
  }

  /// Adds `builder` to this registry. Future calls to `get` and `register_args`
  /// may delegate to `builder` (in whole or in part).
  pub fn register(&mut self, builder: Box<dyn AgentBuilder>) {
    self.agents.insert(builder.name().to_string(), builder);
  }

  /// Constructs an agent from the builder with the given name, using the
  /// command line arguments parsed into `matches` to parameterize agent
  /// behavior.
  pub fn get(&self, name: &str, matches: &ArgMatches) -> Result {
    match self.agents.get(name) {
      Some(builder) => Ok(builder.build(matches)?),
      None => Err(Error::InvalidAgent(name.to_owned())),
    }
  }

  /// Returns the agent identified for the player 1 role.
  pub fn get_player_1_from_arguments(&self, matches: &ArgMatches) -> Result {
    if let Some(agent_name) = matches.value_of(FLAG_PLAYER_1_AGENT) {
      self.get(agent_name, matches)
    } else {
      Err(Error::NoAgentIdentified)
    }
  }

  /// Returns the agent identified for the player 2 role.
  pub fn get_player_2_from_arguments(&self, matches: &ArgMatches) -> Result {
    if let Some(agent_name) = matches.value_of(FLAG_PLAYER_2_AGENT) {
      self.get(agent_name, matches)
    } else {
      Err(Error::NoAgentIdentified)
    }
  }

  /// Adds command-line arguments to `app` for setting the agents that will be
  /// playing. Returns `app`, updated.
  pub fn register_args<'a, 'b>(&'a self, mut app: App<'a, 'b>) -> App<'a, 'b> where 'a: 'b {
    let mut values = Vec::with_capacity(self.agents.len());
    for (_, builder) in self.agents.iter() {
      values.push(builder.name());
      app = builder.register_args(app);
    }
    app
      .arg(Arg::with_name(FLAG_PLAYER_1_AGENT)
           .long(FLAG_PLAYER_1_AGENT)
           .takes_value(true)
           .possible_values(&values[0..values.len()])
           .required(true))
      .arg(Arg::with_name(FLAG_PLAYER_2_AGENT)
           .long(FLAG_PLAYER_2_AGENT)
           .takes_value(true)
           .possible_values(&values[0..values.len()])
           .required(true))
  }
}

/// Constructs an `Agent` that reads moves from a file.
pub struct FileAgentBuilder {
  name: String,
  arg_name: String,
  help: String,
}

impl FileAgentBuilder {
  pub fn new(name: &str) -> Self {
    FileAgentBuilder {
      name: name.to_owned(),
      arg_name: format!("{}_file", name),
      help: format!("File listing moves (one per line) for the agent '{}' to use", name),
    }
  }
}

impl AgentBuilder for FileAgentBuilder {
  fn name(&self) -> &str {
    &self.name
  }

  fn register_args<'a, 'b>(&'a self, app: App<'a, 'b>) -> App<'a, 'b> where 'a: 'b {
    app.arg(Arg::with_name(&self.arg_name)
            .long(&self.arg_name)
            .value_name("FILE")
            .help(&self.help)
            .takes_value(true))
  }

  fn build(&self, matches: &ArgMatches) -> Result {
    if let Some(ref path) = matches.value_of(&self.arg_name) {
      match agent::ReaderAgent::from_file_at(path) {
        Ok(agent) => Ok(Box::new(agent)),
        Err(e) => Err(Error::InvalidAgentParameter { agent: self.name.clone(), parameter: self.arg_name.clone(), error: Some(Box::new(e)), }),
      }
    } else {
      Err(Error::InvalidAgentParameter { agent: self.name.clone(), parameter: self.arg_name.clone(), error: None, })
    }
  }
}

/// Constructs an `Agent` that reads moves from stdin without printing any
/// prompts or game state information.
pub struct StdinAgentBuilder {}

impl StdinAgentBuilder {
  pub fn new() -> Self {
    StdinAgentBuilder{}
  }
}

impl AgentBuilder for StdinAgentBuilder {
  fn name(&self) -> &str {
    "stdin"
  }

  fn register_args<'a, 'b>(&'a self, app: App<'a, 'b>) -> App<'a, 'b> where 'a: 'b {
    app
  }

  fn build(&self, _matches: &ArgMatches) -> Result {
    Ok(Box::new(agent::StdinAgent::with_prompt(|_| "".to_owned())))
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn reader_agents() {
    let mut registry = AgentRegistry::new();
    registry.register(Box::new(FileAgentBuilder::new("file1")));
    registry.register(Box::new(FileAgentBuilder::new("file2")));
    registry.register(Box::new(StdinAgentBuilder::new()));
  }
}
