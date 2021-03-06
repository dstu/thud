use crate::agent_registry::{AgentBuilder, Error};
use clap::{App, Arg, ArgMatches};
use thud_game;

pub struct MctsAgentBuilder {
  name: String,
  iteration_count_flag: String,
  simulation_count_flag: String,
  simulation_thread_limit_flag: String,
  exploration_bias_flag: String,
  compact_graph_flag: String,
  action_selection_flag: String,
  rng_seed_flag: String,
}

impl MctsAgentBuilder {
  pub fn new<S: Into<String>>(name: S) -> Self {
    let name: String = name.into();
    MctsAgentBuilder {
      name: name.clone(),
      iteration_count_flag: format!("{}_iterations", name),
      simulation_count_flag: format!("{}_simulations", name),
      simulation_thread_limit_flag: format!("{}_simulation_threads", name),
      exploration_bias_flag: format!("{}_explore_bias", name),
      compact_graph_flag: format!("{}_compact_search_graph", name),
      action_selection_flag: format!("{}_action_selection", name),
      rng_seed_flag: format!("{}_rng_seed", name),
    }
  }
}

impl AgentBuilder for MctsAgentBuilder {
  fn name(&self) -> &str {
    &self.name
  }

  fn register_args<'a, 'b>(&'a self, app: App<'a, 'b>) -> App<'a, 'b>
  where
    'a: 'b,
  {
    app.arg(Arg::with_name(&self.iteration_count_flag)
            .long(&self.iteration_count_flag)
            .value_name("COUNT")
            .help("Number of iterations of MCTS search for the agent"))
      .arg(Arg::with_name(&self.simulation_count_flag)
           .long(&self.simulation_count_flag)
           .value_name("COUNT")
           .help("Number of simulations to run in MCTS simulation phase for the agent"))
      .arg(Arg::with_name(&self.simulation_thread_limit_flag)
           .long(&self.simulation_thread_limit_flag)
           .value_name("THREADS")
           .help("Maximum number of threads to run MCTS simulations in"))
      .arg(Arg::with_name(&self.exploration_bias_flag)
           .long(&self.exploration_bias_flag)
           .value_name("BIAS")
           .help("UCB1 exploration bias for the agent"))
      .arg(Arg::with_name(&self.compact_graph_flag)
           .long(&self.compact_graph_flag)
           .value_name("PRUNE|CLEAR|RETAIN")
           .help("Search graph compaction for the agent to use between rounds of MCTS"))
      .arg(Arg::with_name(&self.action_selection_flag)
           .long(&self.action_selection_flag)
           .value_name("UCB|VISIT_COUNT")
           .help("Action selection criterion for the agent to use when selecting the action to take after MCTS statistics are gathered"))
      .arg(Arg::with_name(&self.rng_seed_flag)
           .long(&self.rng_seed_flag)
           .value_name("SEED")
           .required(false)
           .help("Hex-valued RNG seed for the agent to use during MCTS"))
  }

  fn build(&self, matches: &ArgMatches) -> crate::agent_registry::Result {
    let simulation_count = match matches
      .value_of(&self.simulation_count_flag)
      .map(|s| s.parse::<u32>())
    {
      Some(Ok(c)) if c > 0 => c,
      Some(Ok(_)) | None => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.simulation_count_flag.clone(),
          error: None,
        })
      }
      Some(Err(e)) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.simulation_count_flag.clone(),
          error: Some(Box::new(e)),
        })
      }
    };
    let simulation_thread_limit = match matches
      .value_of(&self.simulation_thread_limit_flag)
      .map(|s| s.parse::<u32>())
    {
      Some(Ok(c)) if c > 0 => c,
      Some(Ok(_)) | None => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.simulation_thread_limit_flag.clone(),
          error: None,
        })
      }
      Some(Err(e)) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.simulation_thread_limit_flag.clone(),
          error: Some(Box::new(e)),
        })
      }
    };
    let explore_bias = match matches
      .value_of(&self.exploration_bias_flag)
      .map(|s| s.parse::<f64>())
    {
      Some(Ok(b)) if b >= 0.0 => b,
      Some(Ok(_)) | None => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.exploration_bias_flag.clone(),
          error: None,
        })
      }
      Some(Err(e)) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.exploration_bias_flag.clone(),
          error: Some(Box::new(e)),
        })
      }
    };
    let settings = mcts::SearchSettings {
      simulation_count,
      simulation_thread_limit,
      explore_bias,
    };
    let iterations = match matches
      .value_of(&self.iteration_count_flag)
      .map(|s| s.parse::<u32>())
    {
      Some(Ok(c)) if c > 0 => c,
      Some(Ok(_)) | None => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.iteration_count_flag.clone(),
          error: None,
        })
      }
      Some(Err(e)) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.iteration_count_flag.clone(),
          error: Some(Box::new(e)),
        })
      }
    };
    let rng = match matches.value_of(&self.rng_seed_flag) {
      Some(_) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.rng_seed_flag.clone(),
          error: None,
        })
      }
      None => Box::new(rand::rngs::OsRng),
    };
    let action_select = match matches.value_of(&self.action_selection_flag) {
      Some(s) if s.to_lowercase() == "visit_count" => thud_game::ai::mcts::ActionSelect::VisitCount,
      Some(s) if s.to_lowercase() == "ucb" => thud_game::ai::mcts::ActionSelect::Ucb,
      Some(_) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.action_selection_flag.clone(),
          error: None,
        })
      }
      None => thud_game::ai::mcts::ActionSelect::VisitCount,
    };
    let graph_compact = match matches.value_of(&self.compact_graph_flag) {
      Some(s) if s.to_lowercase() == "prune" => thud_game::ai::mcts::GraphCompact::Prune,
      Some(s) if s.to_lowercase() == "clear" => thud_game::ai::mcts::GraphCompact::Clear,
      Some(s) if s.to_lowercase() == "retain" => thud_game::ai::mcts::GraphCompact::Retain,
      Some(_) => {
        return Err(Error::InvalidAgentParameter {
          agent: self.name().into(),
          parameter: self.compact_graph_flag.clone(),
          error: None,
        })
      }
      None => thud_game::ai::mcts::GraphCompact::Prune,
    };
    Ok(Box::new(thud_game::ai::mcts::Agent::new(
      settings,
      iterations,
      rng,
      action_select,
      graph_compact,
    )))
  }
}

#[cfg(test)]
mod test {
  use super::MctsAgentBuilder;
  use crate::agent_registry::AgentBuilder;
  use clap::App;

  #[test]
  fn build_agent() {
    let builder = MctsAgentBuilder::new("mcts");
    let app = builder.register_args(App::new("test"));
    let matches = app
      .get_matches_from_safe(&[
        "bin",
        "--mcts_simulations",
        "5",
        "--mcts_simulation_threads",
        "2",
        "--mcts_iterations",
        "31",
        "--mcts_explore_bias",
        "0.64",
        "--mcts_compact_search_graph",
        "PRUNE",
        "--mcts_action_selection",
        "VISIT_COUNT",
      ])
      .unwrap();
    let _agent = builder.build(&matches).unwrap();
  }
}
