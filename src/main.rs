use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::{Buffer, Rect},
    style::Stylize,
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

#[derive(Debug)]
struct App {
    bits: Bits,
    download_speed_log: Vec<Bits>,
    quit_requested: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            bits: 0.0.into(),
            download_speed_log: Vec::with_capacity(20),
            quit_requested: false,
        }
    }
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
            KeyCode::Char('b') => {
                self.bits.0 += 1.0;
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update(&mut self, delta: Duration) {
        if self.download_speed_log.len() == 20 {
            self.download_speed_log.drain(0..1);
        }
        self.download_speed_log.push(self.bits);
    }

    fn download_speed(&self) -> Bits {
        let summed_difference: f64 = self
            .download_speed_log
            .iter()
            .zip(self.download_speed_log.iter().skip(1))
            .map(|(&Bits(a), &Bits(b))| b - a)
            .sum();
        (summed_difference / (20.0 * App::TICK_RATE.as_secs_f64())).into()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(vec![
            Line::from(format!("{}", self.bits)).centered(),
            Line::from(format!("{}/s", self.download_speed()))
                .dim()
                .centered(),
        ])
        .block(Block::bordered())
        .render(area, buf);
    }
}
