use ::game;
use ::game::board;
use ::clap::{App, Arg};
use ::mcts;

pub const ITERATION_COUNT_FLAG: &'static str = "iterations";
pub const SIMULATION_COUNT_FLAG: &'static str = "simulations";
pub const EXPLORATION_BIAS_FLAG: &'static str = "explore_bias";
pub const INITIAL_BOARD_FLAG: &'static str = "initial_board";
pub const INITIAL_PLAYER_FLAG: &'static str = "initial_player";
pub const AI_PLAYER_FLAG: &'static str = "ai_player";
pub const LOG_LEVEL_FLAG: &'static str = "log_level";

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

pub fn set_common_args<'a, 'b>(mut app: App<'a, 'b>, flags: &[&str]) -> App<'a, 'b> where 'a: 'b {
    let populated_flags: Vec<Arg<'static, 'static>> = flags.iter().map(|f| match *f {
        x if x == ITERATION_COUNT_FLAG =>
            Arg::with_name(ITERATION_COUNT_FLAG)
            .short("i")
            .long("iterations")
            .value_name("ITERATIONS")
            .help("Number of Monte Carlo iterations to run per epoch")
            .takes_value(true)
            .required(true),
        x if x == SIMULATION_COUNT_FLAG =>
            Arg::with_name(SIMULATION_COUNT_FLAG)
            .short("s")
            .long("simulations")
            .value_name("SIMULATIONS")
            .help("Number of simulations to run at each expansion step")
            .takes_value(true)
            .required(true),
        x if x == EXPLORATION_BIAS_FLAG =>
            Arg::with_name(EXPLORATION_BIAS_FLAG)
            .short("b")
            .long("exploration_bias")
            .value_name("BIAS")
            .help("Exploration bias for UCB computation")
            .takes_value(true)
            .required(true),
        x if x == INITIAL_BOARD_FLAG =>
            Arg::with_name(INITIAL_BOARD_FLAG)
            .long("board")
            .possible_values(&["default", "trollendgame", "dwarfendgame"])
            .help("Initial board configuration"),
        x if x == INITIAL_PLAYER_FLAG =>
            Arg::with_name(INITIAL_PLAYER_FLAG)
            .short("p")
            .long("player")
            .possible_values(&["dwarf", "troll"])
            .help("Initial player to play"),
        x if x == AI_PLAYER_FLAG =>
            Arg::with_name(AI_PLAYER_FLAG)
            .short("p")
            .long("player")
            .possible_values(&["dwarf", "troll"])
            .help("Side that the AI will play"),
        x if x == LOG_LEVEL_FLAG =>
            Arg::with_name(LOG_LEVEL_FLAG)
            .long("log_level")
            .possible_values(&["info", "trace", "error", "debug", "off"])
            .help("Logging level"),
        x => panic!("Unrecognized flag identifier '{}'", x),
    }).collect();
    app.args(&populated_flags)
}

pub fn initialize_search(state: mcts::State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}
