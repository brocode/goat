extern crate termion;
extern crate tui;

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::time::{Duration, SystemTime};
use termion::event;
use termion::input::TermRead;
use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Gauge, Paragraph, Widget};

struct AppState {
  size: Rect,
  start_time: SystemTime,
  duration: Duration,
  mappings: BTreeMap<char, KeyMapping>,
  title: String,
}

struct KeyMapping {
  ret_code: i32,
  label: String,
}

impl AppState {
  fn new(duration: Duration, mappings: BTreeMap<char, KeyMapping>, title: String) -> AppState {
    AppState {
      size: Rect::default(),
      start_time: SystemTime::now(),
      duration: duration,
      mappings: mappings,
      title: title,
    }
  }

  fn progress_in_percent(self: &AppState) -> u16 {
    let elapsed: Duration = self
      .start_time
      .elapsed()
      .expect("Expected to determine elapsed time");
    std::cmp::min(
      ((elapsed.as_secs() as f64 / self.duration.as_secs() as f64) * 100.0) as u16,
      100 as u16,
    )
  }

  fn at_end(self: &AppState) -> bool {
    self
      .start_time
      .elapsed()
      .expect("Expected to determine elapsed time") > self.duration
  }
}

enum Event {
  Input(event::Key),
  Tick,
}

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

fn parse_mappings(raw_mappings: Vec<String>) -> Result<BTreeMap<char, KeyMapping>, String> {
  let mut mappings: BTreeMap<char, KeyMapping> = BTreeMap::new();
  for mapping in raw_mappings {
    let mut split: Vec<&str> = mapping.split(":").collect();
    if split.len() == 3 {
      if let Some(char) = split[1].chars().next() {
        if let Ok(ret_code) = split[0].parse::<i32>() {
          if ret_code > 113 || ret_code < 64 {
            return Err(format!(
              "Invalid mapping '{}', retcode should be < 64 or > 113",
              mapping
            ));
          }
          mappings.insert(
            char,
            KeyMapping {
              ret_code: ret_code,
              label: split[2].to_string(),
            },
          );
        } else {
          return Err(format!(
            "Invalid mapping '{}', retcode should be a number",
            mapping
          ));
        }
      } else {
        return Err(format!(
          "Invalid mapping '{}', keycode should be a char",
          mapping
        ));
      }
    } else {
      return Err(format!(
        "Invalid mapping '{}', format should be <retcode>:<key>:<label>",
        mapping
      ));
    }
  }
  mappings.insert(
    'q',
    KeyMapping {
      ret_code: 1,
      label: "abort".to_string(),
    },
  );
  mappings.insert(
    'c',
    KeyMapping {
      ret_code: 0,
      label: "continue".to_string(),
    },
  );
  Ok(mappings)
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
  match parse_mappings(raw_mappings) {
    Ok(mappings) => {
      let return_code = {
        let mut app_state = AppState::new(Duration::from_secs(time as u64), mappings, title);
        let backend = MouseBackend::new().expect("Expected to initialize backend");
        let mut terminal = Terminal::new(backend).expect("Expected to initialize terminal");

        let return_code = run(&mut terminal, app_state);
        terminal.show_cursor().expect("Expected to show cursor");
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

fn run(terminal: &mut Terminal<MouseBackend>, mut app_state: AppState) -> i32 {
  // Channels
  let (tx, rx) = mpsc::channel();
  let input_tx = tx.clone();
  let clock_tx = tx.clone();

  // Input
  thread::spawn(move || {
    let stdin = io::stdin();
    for c in stdin.keys() {
      let evt = c.unwrap();
      input_tx.send(Event::Input(evt)).unwrap();
    }
  });

  // Tick
  thread::spawn(move || loop {
    clock_tx.send(Event::Tick).unwrap();
    thread::sleep(time::Duration::from_millis(500));
  });

  // First draw call
  terminal.clear().unwrap();
  terminal.hide_cursor().unwrap();
  app_state.size = terminal.size().unwrap();
  draw(terminal, &app_state);

  loop {
    let size = terminal.size().unwrap();
    if size != app_state.size {
      terminal.resize(size).unwrap();
      app_state.size = size;
    }

    let evt = rx.recv().unwrap();
    match evt {
      Event::Input(input) => if let event::Key::Char(key) = input {
        if let Entry::Occupied(value) = app_state.mappings.entry(key) {
          return value.get().ret_code;
        }
      },
      Event::Tick => {
        if app_state.at_end() {
          break;
        }
      }
    }
    draw(terminal, &app_state);
  }

  0
}

fn draw(t: &mut Terminal<MouseBackend>, app_state: &AppState) {
  let mut text = String::with_capacity(100);
  for (key, value) in app_state.mappings.iter() {
    text.push_str(&format!("{{fg=green {}}} -> {}\n", key, value.label))
  }
  Group::default()
    .direction(Direction::Vertical)
    .margin(2)
    .sizes(&[
      Size::Percent(20),
      Size::Percent(20),
      Size::Percent(10),
      Size::Percent(50),
    ])
    .render(t, &app_state.size, |t, chunks| {
      Paragraph::default()
        .block(
          Block::default()
            .borders(Borders::ALL)
            .title(&app_state.title)
            .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::Bold)),
        )
        .wrap(true)
        .text(&text)
        .render(t, &chunks[0]);
      Gauge::default()
        .block(Block::default().title("timer").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .percent(app_state.progress_in_percent())
        .label(&format!("{}/100", app_state.progress_in_percent()))
        .render(t, &chunks[1]);
    });

  t.draw().unwrap();
}
