use crate::keymapping::KeyMapping;

use std::collections::BTreeMap;
use std::io;
use std::io::Stdout;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::time::{Duration, SystemTime};
use termion::event;
use termion::input::TermRead;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use tui::Terminal;

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

const NANOS_PER_MILLI: u64 = 1_000_000;
const MILLIS_PER_SEC: u64 = 1_000;

fn duration_as_millis(duration: &Duration) -> u64 {
  let sub_millis = u64::from(duration.subsec_nanos()) / NANOS_PER_MILLI;
  duration.as_secs() * MILLIS_PER_SEC + sub_millis
}

impl AppState {
  pub fn new(duration: Duration, mappings: BTreeMap<char, KeyMapping>, title: String) -> AppState {
    AppState {
      size: Rect::default(),
      start_time: SystemTime::now(),
      duration,
      mappings,
      title,
    }
  }

  fn time_passed(self: &AppState) -> Duration {
    self.start_time.elapsed().expect("Expected to determine elapsed time")
  }

  fn time_passed_in_seconds(self: &AppState) -> u64 {
    self.time_passed().as_secs() as u64
  }

  fn progress_in_percent(self: &AppState) -> u16 {
    let elapsed: Duration = self.time_passed();
    std::cmp::min(
      ((duration_as_millis(&elapsed) as f64 / duration_as_millis(&self.duration) as f64) * 100.0) as u16,
      100_u16,
    )
  }

  fn at_end(self: &AppState) -> bool {
    self.start_time.elapsed().expect("Expected to determine elapsed time") > self.duration
  }
}

pub fn run(terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>, mut app_state: AppState) -> i32 {
  // Channels
  let (tx, rx) = mpsc::channel();
  let input_tx = tx.clone();
  let clock_tx = tx;

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
      Event::Input(input) => {
        if let event::Key::Char(key) = input {
          if let Some(value) = app_state.mappings.get(&key) {
            return value.ret_code;
          }
        }
      }
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

fn draw(t: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>, app_state: &AppState) {
  let mut text: Vec<Spans> = Vec::new();
  for (key, value) in &app_state.mappings {
    text.push(Spans::from(vec![
      Span::styled(key.to_string(), Style::default().fg(Color::Green)),
      Span::raw(format!(" -> {}\n", value.label)),
    ]))
  }

  t.draw(|f| {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(2)
      .constraints(vec![Constraint::Percentage(72), Constraint::Percentage(25)])
      .split(app_state.size);

    let para = Paragraph::new(Text { lines: text })
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title(Span::styled(&app_state.title, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
      )
      .wrap(Wrap { trim: true });
    let label = format!("{}s / {}s", app_state.time_passed_in_seconds(), app_state.duration.as_secs());
    let gauge = Gauge::default()
      .block(Block::default().title("timer").borders(Borders::ALL))
      .gauge_style(Style::default().fg(Color::Cyan))
      .percent(app_state.progress_in_percent())
      .label(Span::raw(&label));

    f.render_widget(para, chunks[0]);
    f.render_widget(gauge, chunks[1]);
  })
  .expect("Expected to be able to draw on terminal");
}
