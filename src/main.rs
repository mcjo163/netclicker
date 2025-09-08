use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::bits::Bits;

pub mod bits;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();

    result
}

#[derive(Debug, Default)]
struct App {
    bits: Bits,
    bit_rate: f64,
    quit_requested: bool,
}

impl App {
    /// Game updates and event polling are subject to this rate.
    const TICK_RATE: Duration = Duration::from_millis(1000 / 20);

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last_tick = Instant::now();

        loop {
            let timeout = App::TICK_RATE.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(evt) if evt.kind == KeyEventKind::Press => {
                        self.handle_key_event(evt);
                    }
                    _ => {}
                }
            }

            if self.quit_requested {
                return Ok(());
            }

            terminal.draw(|frame| self.draw(frame))?;

            if last_tick.elapsed() >= App::TICK_RATE {
                self.update(last_tick.elapsed());
                last_tick = Instant::now();
            }
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit_requested = true;
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update(&mut self, delta: Duration) {
        if self.bit_rate == 0.0 {
            self.bit_rate = 1.0;
        } else {
            self.bit_rate *= 1.6;
        }

        self.bits.0 += self.bit_rate * delta.as_secs_f64();
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(format!("{}", self.bits)).centered();
        Paragraph::new(line)
            .block(Block::bordered())
            .render(area, buf);
    }
}
