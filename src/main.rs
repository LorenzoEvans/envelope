use envelope::app::{App, AppResult};
use envelope::event::{Event, EventHandler};
use envelope::handler::handle_key_events;
use envelope::tui::Tui;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> AppResult<()> {
    let mut app = App::new();
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let app_result = loop {
        if let Err(err) = tui.draw(&mut app) {
            break Err(err);
        }

        let event = match tui.events.next() {
            Ok(event) => event,
            Err(err) => break Err(err),
        };

        let handle_result = match event {
            Event::Key(key_event) => handle_key_events(key_event, &mut app),
            Event::Mouse(_) | Event::Resize(_, _) => Ok(()),
        };

        if let Err(err) = handle_result {
            break Err(err);
        }

        if !app.running {
            break Ok(());
        }
    };

    let exit_result = tui.exit();
    app_result?;
    exit_result?;
    Ok(())
}
