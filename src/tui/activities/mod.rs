pub mod add_new_command;
pub mod command_browser;
pub mod keybind_help;

use std::sync::{Arc, Mutex};

use anyhow as ah;
use add_new_command::AddNewCommand;
use command_browser::CommandBrowser;
use keybind_help::KeybindHelp;
use ratatui::{prelude::TermionBackend, Terminal};
use termion::{event::Key, screen::AlternateScreen};

use super::event_loop::Event;

type TermHandle =
    Terminal<TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>>;

pub trait Activity {
    fn on_key_press(&mut self, key: Key);
    fn draw(&mut self, terminal: &mut TermHandle);
    fn signal_event_loop(&self, event: Event) -> ah::Result<()>;
}

pub enum Activities {
    AddNewCommand(Arc<Mutex<AddNewCommand>>),
    CommandBrowser(Arc<Mutex<CommandBrowser>>),
    KeybindHelp(Arc<Mutex<KeybindHelp>>),
}

impl Activity for Activities {
    fn on_key_press(&mut self, key: Key) {
        match self {
            Activities::AddNewCommand(activity) => activity.lock().unwrap().on_key_press(key),
            Activities::CommandBrowser(activity) => activity.lock().unwrap().on_key_press(key),
            Activities::KeybindHelp(activity) => activity.lock().unwrap().on_key_press(key),
        }
    }

    fn draw(&mut self, terminal: &mut TermHandle) {
        match self {
            Activities::AddNewCommand(activity) => activity.lock().unwrap().draw(terminal),
            Activities::CommandBrowser(activity) => activity.lock().unwrap().draw(terminal),
            Activities::KeybindHelp(activity) => activity.lock().unwrap().draw(terminal),
        }
    }

    fn signal_event_loop(&self, event: Event) -> ah::Result<()> {
        match self {
            Activities::AddNewCommand(activity) => activity.lock().unwrap().signal_event_loop(event),
            Activities::CommandBrowser(activity) => activity.lock().unwrap().signal_event_loop(event),
            Activities::KeybindHelp(activity) => activity.lock().unwrap().signal_event_loop(event),
        }
    }
}
