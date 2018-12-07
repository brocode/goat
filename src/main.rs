extern crate termion;
extern crate tui;

#[macro_use]
extern crate clap;

extern crate atty;

use crate::keymapping::parse_mappings;
use crate::ui::{run, AppState};
use clap::{App, AppSettings, Arg};
use std::io;
use std::time::Duration;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

mod keymapping;
mod ui;

fn app<'a>() -> App<'a, 'a> {
  App::new("goat")
    .version(crate_version!())
    .author("Brocode <bros@brocode.sh>")
    .about("better sleep")
    .global_setting(AppSettings::ColoredHelp)
    .arg(
      Arg::with_name("time")
        .short("t")
        .long("time")
        .takes_value(true)
        .number_of_values(1)
        .help("timer in seconds")
        .required(true),
    )
    .arg(
      Arg::with_name("title")
        .long("title")
        .takes_value(true)
        .number_of_values(1)
        .help("title")
        .required(false),
    )
    .arg(
      Arg::with_name("mappings")
        .short("m")
        .long("mapping")
        .takes_value(true)
        .multiple(true)
        .number_of_values(1)
        .help("Keybinding mapping. Format: <retcode>:<key>:<label> (64 <= retcode <= 113)")
        .required(false),
    )
}

fn main() {
  let matches = app().get_matches();
  let time: i32 = matches
    .value_of("time")
    .expect("clap should ensure this")
    .parse::<i32>()
    .expect("Expected time to be a valid number");
  let title: String = matches.value_of("title").unwrap_or("GOAT").to_string();
  let raw_mappings = matches.values_of_lossy("mappings").unwrap_or_default();
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
