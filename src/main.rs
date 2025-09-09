use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph, Widget},
};

use crate::bits::{BitLog, Bits};

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
    bit_log: BitLog<{ App::TICKS_PER_SECOND * 30 }>,
    quit_requested: bool,
}

impl App {
    /// Game updates and event polling are subject to this rate.
    const TICKS_PER_SECOND: usize = 20;
    const TICK_RATE: Duration = Duration::from_millis(1000 / App::TICKS_PER_SECOND as u64);

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

    fn update(&mut self, _delta: Duration) {
        self.bit_log.track(self.bits);
    }

    fn download_speed(&self) -> Bits {
        // The raw diff of bit count is the average per second, since the length
        // of the log matches the number of ticks per second.
        self.bit_log.diff(App::TICKS_PER_SECOND - 1)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let app_layout = Layout::horizontal(Constraint::from_percentages([60, 40])).split(area);
        let right_inner_layout =
            Layout::vertical(Constraint::from_percentages([30, 70])).split(app_layout[1]);

        // Bit log chart: app_layout[0]
        let raw_data: Vec<_> = self
            .bit_log
            .to_vec()
            .iter()
            .rev()
            .zip((0..).map(|x| -(x as f64 / App::TICKS_PER_SECOND as f64)))
            .map(|(&Bits(y), x)| (x, y))
            .collect();

        let dataset = Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().blue())
            .data(&raw_data[..]);

        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([-30.0, 0.0])
            .labels(["30s", "15s", "now"]);

        let y_axis = Axis::default()
            .title("data")
            .style(Style::default().white())
            .bounds([0.0, 10.0_f64.max(self.bits.0 * 1.2)]);

        Chart::new(vec![dataset])
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(app_layout[0], buf);

        // Current stats: right_inner_layout[0]
        Paragraph::new(vec![
            Line::from(format!("{}", self.bits)).centered(),
            Line::from(format!("{}ps", self.download_speed()))
                .dim()
                .centered(),
        ])
        .render(right_inner_layout[0], buf);

        // "Shop": right_inner_layout[1]
        Block::bordered()
            .title(" shop ".bold().into_centered_line())
            .render(right_inner_layout[1], buf);
    }
}
