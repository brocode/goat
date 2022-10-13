extern crate termion;
extern crate tui;

use crate::keymapping::parse_mappings;
use crate::ui::{run, AppState};
use clap::{crate_version, Command, Arg, ArgAction, value_parser};
use std::io;
use std::time::Duration;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

mod keymapping;
mod ui;

fn app() -> Command {
  Command::new("goat")
    .version(crate_version!())
    .author("Brocode <bros@brocode.sh>")
    .about("better sleep")
    .arg(
      Arg::new("time")
        .short('t')
        .long("time")
        .num_args(1)
        .value_parser(value_parser!(u32))
        .help("timer in seconds")
        .required(true),
    )
    .arg(
      Arg::new("title")
        .long("title")
        .default_value("GOAT")
        .num_args(1)
        .help("title")
        .required(false),
    )
    .arg(
      Arg::new("mappings")
        .short('m')
        .long("mapping")
        .action(ArgAction::Append)
        .num_args(1)
        .help("Keybinding mapping. Format: <retcode>:<key>:<label> (64 <= retcode <= 113)")
        .required(false),
    )
}

fn main() {
  let matches = app().get_matches();
  let time: u32 = matches
    .get_one::<u32>("time")
    .expect("Expected time to be a valid number")
    .to_owned();
  let title: String = matches.get_one::<String>("title").expect("Clap ensures this").to_owned();
  let raw_mappings = matches.get_many::<String>("mappings")
    .unwrap_or_default()
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();
  if !atty::is(atty::Stream::Stdin) || !atty::is(atty::Stream::Stdout) {
    println!("goat - sleeping for {} seconds: '{}'", time, title);
    std::thread::sleep(Duration::from_secs(time as u64));
  } else {
    match parse_mappings(raw_mappings) {
      Ok(mappings) => {
        let return_code = {
          let app_state = AppState::new(Duration::from_secs(time as u64), mappings, title);
          let stdout = io::stdout().into_raw_mode().expect("Expected to initialize out stream.");
          let backend = TermionBackend::new(stdout);
          let mut terminal = Terminal::new(backend).expect("Expected to initialize terminal");

          let return_code = run(&mut terminal, app_state);
          terminal.show_cursor().expect("Expected to show cursor");
          terminal.clear().expect("Expected to clear terminal");
          return_code
        };
        std::process::exit(return_code)
      }
      Err(message) => {
        eprintln!("{}", message);
        std::process::exit(1)
      }
    }
  }
}
