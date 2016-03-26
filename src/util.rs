use ::clap::{App, Arg};

pub const ITERATION_COUNT_FLAG: &'static str = "iterations";
pub const SIMULATION_COUNT_FLAG: &'static str = "simulations";
pub const EXPLORATION_BIAS_FLAG: &'static str = "explore_bias";

pub fn set_common_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> where 'a: 'b {
    app.args(&[
        Arg::with_name(ITERATION_COUNT_FLAG)
            .short("i")
            .long("iterations")
            .value_name("ITERATIONS")
            .help("Number of MCTS iterations to run")
            .takes_value(true)
            .required(true),
        Arg::with_name(SIMULATION_COUNT_FLAG)
            .short("s")
            .long("simulations")
            .value_name("SIMULATIONS")
            .help("Number of simulations to run at each expansion step")
            .takes_value(true)
            .required(true),
        Arg::with_name(EXPLORATION_BIAS_FLAG)
            .short("b")
            .long("exploration_bias")
            .value_name("BIAS")
            .help("Exploration bias for UCB computation")
            .takes_value(true)
            .required(true)
            ])
}
