use ::game::board;
use ::clap::{App, Arg};

pub const ITERATION_COUNT_FLAG: &'static str = "iterations";
pub const SIMULATION_COUNT_FLAG: &'static str = "simulations";
pub const EXPLORATION_BIAS_FLAG: &'static str = "explore_bias";
pub const INITIAL_BOARD_FLAG: &'static str = "initial_board";
pub const INITIAL_PLAYER_FLAG: &'static str = "initial_player";

arg_enum! {
    #[derive(Debug)]
    pub enum InitialBoard {
        Default,
        TrollEndgame,
        DwarfEndgame
    }
}

impl InitialBoard {
    pub fn cells(&self) -> board::Cells {
        match *self {
            InitialBoard::Default => board::decode_board(DEFAULT_CELLS),
            InitialBoard::TrollEndgame => board::decode_board(TROLL_ENDGAME),
            InitialBoard::DwarfEndgame => board::decode_board(DWARF_ENDGAME),
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
            .required(true),
        Arg::with_name(INITIAL_BOARD_FLAG)
            .long("board")
            .value_name("default|trollendgame|dwarfendgame")
            .help("Initial board configuration"),
        Arg::with_name(INITIAL_PLAYER_FLAG)
            .short("p")
            .long("player")
            .value_name("dwarf|troll")
            .help("Player to play"),
        ])
}
