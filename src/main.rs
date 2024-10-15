use crate::tui::{App, AppResult, Config, Tui};

use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::{
    command::{Command, InputCommand},
    event::{Event, EventHandler},
};

mod args;
mod aws;
mod tui;

#[tokio::main]
async fn main() -> AppResult<()> {
    // trace::initialize_logging()?;

    // trace_dbg!("Starting elysium");
    let args = args::Args::parse();
    let mut aws = aws::AwsCloud::new(&args.profile, &args.region).await?;
    aws.load().await?;
    
    let cfg = Config::load();
    let mut app = App::new(aws, cfg)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while !app.should_quit {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => {
                let command = if app.input_mode {
                    Command::Input(InputCommand::parse(key_event, &app.input))
                } else {
                    Command::from(key_event)
                };
                app.run_command(command, tui.events.sender.clone())?;    
            }
            Event::Mouse(mouse_event) => {
                app.run_command(Command::from(mouse_event), tui.events.sender.clone())?;
            }
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;
    Ok(())
}