use chrono;
use clap::{App, Arg};
use fern;
use log;
use std::io;

pub const FLAG_INITIAL_BOARD: &'static str = "initial_board";
pub const FLAG_INITIAL_PLAYER: &'static str = "initial_player";
pub const FLAG_LOG_LEVEL: &'static str = "log_level";

pub fn with_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b>
where
  'a: 'b,
{
  app
    .arg(
      Arg::with_name(FLAG_INITIAL_BOARD)
        .long("board")
        .takes_value(true)
        .required(true)
        .possible_values(&["default", "trollendgame", "dwarfendgame", "dwarfboxed"])
        .help("Initial board configuration"),
    )
    .arg(
      Arg::with_name(FLAG_INITIAL_PLAYER)
        .long("initial-player")
        .takes_value(true)
        .possible_values(&["dwarf", "troll"])
        .required(true)
        .help("The player who will go first"),
    )
    .arg(
      Arg::with_name(FLAG_LOG_LEVEL)
        .long("log-level")
        .takes_value(true)
        .possible_values(&["info", "trace", "error", "debug", "off"])
        .help("Logging level"),
    )
}

pub fn init_logger(logging_level: log::LevelFilter) {
  if let Err(e) = fern::Dispatch::new()
    .format(|out, message, record| {
      let time = chrono::Local::now().format("%Y-%m-%d %T%.3f%z").to_string();
      out.finish(format_args!(
        "[{}][{}]@{} {}",
        record.level(),
        record.target(),
        time,
        message
      ))
    })
    .level(logging_level)
    .chain(io::stderr())
    .apply()
  {
    panic!("Filed to initialize global logger: {}", e);
  }
}
