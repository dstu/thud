use clap::{self, arg_enum};
use thud_game::{self, board};

// pub use thud_game::ai::mcts::deconvolve_transpositions::Game as ThudGame;
// pub use thud_game::ai::mcts::deconvolve_transpositions::Payoff as ThudPayoff;
// pub use thud_game::ai::mcts::deconvolve_transpositions::State as ThudState;
// pub use thud_game::ai::mcts::deconvolve_transpositions::Statistics as ThudStatistics;

pub mod agent_registry;
pub mod init;

pub const FLAG_INITIAL_BOARD: &'static str = "initial_board";
pub const FLAG_INITIAL_PLAYER: &'static str = "initial_player";
pub const FLAG_LOG_LEVEL: &'static str = "log_level";

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

/// Adds arguments to `app` that are common across all user-facing UIs. These
/// arguments come from constants in this module. Returns the updated `App`.
pub fn set_args<'a, 'b>(app: clap::App<'a, 'b>, flags: &[&str]) -> clap::App<'a, 'b>
where
  'a: 'b,
{
  let populated_flags: Vec<clap::Arg<'static, 'static>> = flags
    .iter()
    .map(|f| match *f {
      x if x == FLAG_INITIAL_BOARD => clap::Arg::with_name(FLAG_INITIAL_BOARD)
        .long("board")
        .takes_value(true)
        .possible_values(&["default", "trollendgame", "dwarfendgame", "dwarfboxed"])
        .help("Initial board configuration"),
      x if x == FLAG_INITIAL_PLAYER => clap::Arg::with_name(FLAG_INITIAL_PLAYER)
        .short("p")
        .long("player")
        .takes_value(true)
        .possible_values(&["dwarf", "troll"])
        .help("Initial player to play"),
      x if x == FLAG_LOG_LEVEL => clap::Arg::with_name(FLAG_LOG_LEVEL)
        .long("log_level")
        .takes_value(true)
        .possible_values(&["info", "trace", "error", "debug", "off"])
        .help("Logging level"),
      x => panic!("Unrecognized flag identifier '{}'", x),
    })
    .collect();
  app.args(&populated_flags)
}
