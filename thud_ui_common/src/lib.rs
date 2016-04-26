extern crate chrono;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate fern;
extern crate thud_game;

use clap::{App, Arg};
use thud_game::board;

pub const ITERATION_COUNT_FLAG: &'static str = "iterations";
pub const SIMULATION_COUNT_FLAG: &'static str = "simulations";
pub const EXPLORATION_BIAS_FLAG: &'static str = "explore_bias";
pub const INITIAL_BOARD_FLAG: &'static str = "initial_board";
pub const INITIAL_PLAYER_FLAG: &'static str = "initial_player";
pub const AI_PLAYER_FLAG: &'static str = "ai_player";
pub const LOG_LEVEL_FLAG: &'static str = "log_level";
pub const MOVE_SELECTION_CRITERION_FLAG: &'static str = "move_selection_criterion";
pub const RNG_SEED_FLAG: &'static str = "rng_seed";
pub const COMPACT_SEARCH_GRAPH_FLAG: &'static str = "compact_search_graph";

pub use thud_game::ai::mcts::allow_transpositions::Game as ThudGame;
pub use thud_game::ai::mcts::allow_transpositions::Payoff as ThudPayoff;
pub use thud_game::ai::mcts::allow_transpositions::State as ThudState;
pub use thud_game::ai::mcts::allow_transpositions::Statistics as ThudStatistics;

arg_enum! {
    #[derive(Debug)]
    pub enum InitialBoard {
        Default,
        TrollEndgame,
        DwarfEndgame,
        DwarfBoxed
    }
}

impl InitialBoard {
    pub fn cells(&self) -> board::Cells {
        match *self {
            InitialBoard::Default => board::decode_board(DEFAULT_CELLS),
            InitialBoard::TrollEndgame => board::decode_board(TROLL_ENDGAME),
            InitialBoard::DwarfEndgame => board::decode_board(DWARF_ENDGAME),
            InitialBoard::DwarfBoxed => board::decode_board(DWARF_BOXED),
        }
    }
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum MoveSelectionCriterion {
        VisitCount,
        Ucb
    }
}

const DEFAULT_CELLS: &'static str = r#"
.....dd_dd.....
....d_____d....
...d_______d...
..d_________d..
.d___________d.
d_____________d
d_____TTT_____d
______TOT______
d_____TTT_____d
d_____________d
.d___________d.
..d_________d..
...d_______d...
....d_____d....
.....dd_dd.....
"#;

const TROLL_ENDGAME: &'static str = r#"
....._____.....
...._______....
...__ddd____...
..___d_d_____..
.____d_d______.
_______________
______T________
______TO_______
______T________
_______________
._____________.
..___________..
..._________...
...._______....
....._____.....
"#;

const DWARF_ENDGAME: &'static str = r#"
....._____.....
...._______....
..._________...
..___________..
._____________.
_______________
_______________
_ddd__TO_______
_______________
_______________
._____________.
..___________..
..._________...
...._______....
....._____.....
"#;

const DWARF_BOXED: &'static str = r#"
.....d___d.....
...._______....
..._________...
..___________..
._d_________d_.
_dd_________dd_
dddd__TTT__dddd
______TOT______
dddd__TTT__dddd
_dd_________dd_
._d_________d_.
..d_________d..
..._________...
...._______....
.....d___d.....
"#;

pub fn set_args<'a, 'b>(app: App<'a, 'b>, flags: &[&str]) -> App<'a, 'b> where 'a: 'b {
    let populated_flags: Vec<Arg<'static, 'static>> = flags.iter().map(|f| match *f {
        x if x == ITERATION_COUNT_FLAG =>
            Arg::with_name(ITERATION_COUNT_FLAG)
            .short("i")
            .long("iterations")
            .takes_value(true)
            .value_name("ITERATIONS")
            .help("Number of Monte Carlo iterations to run per epoch")
            .required(true),
        x if x == SIMULATION_COUNT_FLAG =>
            Arg::with_name(SIMULATION_COUNT_FLAG)
            .short("s")
            .long("simulations")
            .takes_value(true)
            .value_name("SIMULATIONS")
            .help("Number of simulations to run at each expansion step")
            .required(true),
        x if x == EXPLORATION_BIAS_FLAG =>
            Arg::with_name(EXPLORATION_BIAS_FLAG)
            .short("b")
            .long("exploration_bias")
            .takes_value(true)
            .value_name("BIAS")
            .help("Exploration bias for UCB computation")
            .required(true),
        x if x == INITIAL_BOARD_FLAG =>
            Arg::with_name(INITIAL_BOARD_FLAG)
            .long("board")
            .takes_value(true)
            .possible_values(&["default", "trollendgame", "dwarfendgame", "dwarfboxed"])
            .help("Initial board configuration"),
        x if x == INITIAL_PLAYER_FLAG =>
            Arg::with_name(INITIAL_PLAYER_FLAG)
            .short("p")
            .long("player")
            .takes_value(true)
            .possible_values(&["dwarf", "troll"])
            .help("Initial player to play"),
        x if x == AI_PLAYER_FLAG =>
            Arg::with_name(AI_PLAYER_FLAG)
            .short("p")
            .long("player")
            .takes_value(true)
            .possible_values(&["dwarf", "troll"])
            .help("Side that the AI will play"),
        x if x == LOG_LEVEL_FLAG =>
            Arg::with_name(LOG_LEVEL_FLAG)
            .long("log_level")
            .takes_value(true)
            .possible_values(&["info", "trace", "error", "debug", "off"])
            .help("Logging level"),
        x if x == MOVE_SELECTION_CRITERION_FLAG =>
            Arg::with_name(MOVE_SELECTION_CRITERION_FLAG)
            .long("move_selection")
            .takes_value(true)
            .possible_values(&["visitcount", "ucb"])
            .help("Criteria for selecting the best move to make"),
        x if x == RNG_SEED_FLAG =>
            Arg::with_name(RNG_SEED_FLAG)
            .long("seed")
            .takes_value(true)
            .help("Manually specify RNG seed"),
        x if x == COMPACT_SEARCH_GRAPH_FLAG =>
            Arg::with_name(COMPACT_SEARCH_GRAPH_FLAG)
            .long("compact_graph")
            .takes_value(false)
            .help("Compact the search graph after each move"),
        x => panic!("Unrecognized flag identifier '{}'", x),
    }).collect();
    app.args(&populated_flags)
}

pub fn init_logger(logging_level: log::LogLevelFilter) {
    let config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            let time = chrono::Local::now().format("%Y-%m-%d %T%.3f%z").to_string();
            format!("[{}][{}] {}", time, level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: logging_level,
    };
    if let Err(e) = fern::init_global_logger(config, log::LogLevelFilter::Trace) {
        panic!("Filed to initialize global logger: {}", e);
    }
}
