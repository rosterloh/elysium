use crate::tui::{App, AppResult, Config};

use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::{
    event::{Event, EventHandler},
    terminal::UserInterface,
    update,
};

mod args;
mod devices;
mod tui;

#[tokio::main]
async fn main() -> AppResult<()> {
    // trace::initialize_logging()?;

    // trace_dbg!("Starting elysium");
    let args = args::Args::parse();
    let items = devices::config::load(&args.profile, &args.region).await?;
    
    let cfg = Config::load();
    let mut app = App::new(cfg, items);

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = UserInterface::new(terminal, events);

    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;

    // if let Some(task) = app.task_to_exec {
    //     return taskfile::command::run_task(task.name);
    // }

    Ok(())
}