use keymapping::KeyMapping;
use std;
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

pub struct AppState {
  pub size: Rect,
  pub start_time: SystemTime,
  pub duration: Duration,
  pub mappings: BTreeMap<char, KeyMapping>,
  pub title: String,
}

enum Event {
  Input(event::Key),
  Tick,
}

impl AppState {
  pub fn new(duration: Duration, mappings: BTreeMap<char, KeyMapping>, title: String) -> AppState {
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

pub fn run(terminal: &mut Terminal<MouseBackend>, mut app_state: AppState) -> i32 {
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
