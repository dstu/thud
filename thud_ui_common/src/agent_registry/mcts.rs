use rand::Rng;
use std::marker::Send;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use thud_game;

pub struct MctsAgentBuilder {
  name: String,
}

impl MctsAgentBuilder {
  pub fn new(name: String) -> Self {
    MctsAgentBuilder { name }
  }

  fn iteration_count_flag(&self) -> String {
    format!("{}_iterations", self.name)
  }

  fn exploration_bias_flag(&self) -> String {
    format!("{}_explore_bias", self.name)
  }

  fn ai_player_flag(&self) -> String {
    format!("{}_ai_player", self.name)
  }

  fn compact_search_graph_flag(&self) -> String {
    format!("{}_compact_search_graph", self.name)
  }

  fn action_selection_flag(&self) -> String {
    format!("{}_action_selection", self.name)
  }

  fn rng_seed_flag(&self) -> String {
    format!("{}_rng_seed", self.name)
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
    app.arg(Arg::with_name(self.iteration_count_flag())
            .long(self.iteration_count_flag())
            .value_name("COUNT")
            .help(format!("Number of iterations of MCTS search for the agent '{}' to use", self.name)))
      .arg(Arg::with_name(self.exploration_bias_flag())
           .long(self.exploration_bias_flag())
           .value_name("BIAS")
           .help(format!("UCB1 exploration bias for the agent '{}' to use", self.name)))
      .arg(Arg::with_name(self.ai_player_flag())
           .long(self.ai_player_flag())
           .value_name("TROLL|DWARF")
           .help(format!("Player for the agent '{}' to play as", self.name)))
      .arg(Arg::with_name(self.compact_search_graph_flag())
           .long(self.compact_search_graph_flag())
           .value_name("PRUNE|CLEAR|RETAIN")
           .help(format!("Search graph compaction for the agent '{}' to use between rounds of MCTS", self.name)))
      .arg(Arg::with_name(self.action_selection_criterion_flag())
           .long(self.action_selection_criterion_flag())
           .value_name("UCB|VISIT_COUNT")
           .help(format!("Action selection criterion for the agent '{}' to use when selecting the action to take after MCTS statistics are gathered", self.name)))
      .arg(Arg::with_name(self.rng_seed_flag())
           .long(self.rng_seed_flag())
           .value_name("SEED")
           .help(format!("Hex-valued RNG seed for the agent '{}' to use during MCTS", self.name)))
  }

  fn build(&self, matches: &ArgMatches) -> crate::agent_registry::Result {
    let settings = unimplemented!();
    let iterations = unimplemented!();
    let player = unimplemented!();
    let rng = unimplemented!();
    let action_select = unimplemented!();
    let graph_compact = unimplemented!();
    Ok(thud_game::ai::mcts::Agent::new(
      settings,
      iterations,
      player,
      rng,
      action_select,
      graph_compact,
    ))
  }
}
