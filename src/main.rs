use color_eyre::Result;
use ratatui::DefaultTerminal;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();

    result
}

#[derive(Debug, Default)]
struct App {
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        Ok(())
    }
}
