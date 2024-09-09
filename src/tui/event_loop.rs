use crossbeam_channel::{unbounded, Receiver, Sender};
use ratatui::{backend::TermionBackend, Terminal};

use termion::event::Key;
use termion::input::TermRead;

use super::activities::keybind_help::KeybindHelp;
use std::io::stdout;
use std::process::exit;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::Mutex;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use anyhow as ah;
use std::sync::Arc;

use crate::data::models::Command;
use crate::data::storage::CommandDatabase;

use crate::tui::{
    activities::Activities,
    activities::Activity,
    activities::{
        add_new_command::AddNewCommand, command_browser::CommandBrowser,
    },
};

pub enum Event {
    KeyPressed(Key),
    DatabaseUpdate(Command),
    DatabaseDelete(Command),
    Tick,
}

pub struct EventLoop {
    // Activity instances.
    command_browser: Arc<Mutex<CommandBrowser>>,
    add_new_command: Arc<Mutex<AddNewCommand>>,
    keybind_help: Arc<Mutex<KeybindHelp>>,

    // The current activity that is being displayed.
    current_activity: Option<Activities>,

    tick_rate: Arc<AtomicU64>,

    // Kill switch for threads.
    kill_switch: Arc<AtomicBool>,

    // Channels for sending events to the event loop, and receiving events.
    event_receiver: Receiver<Event>,
    event_sender: Sender<Event>,

    // Channel for receiving tick events from the tick thread.
    join_handles: Vec<JoinHandle<()>>,

    command_database: CommandDatabase,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop {
    pub fn new() -> Self {
        let command_database = CommandDatabase::new().expect(
            "Could not start event loop, as the command database couldn't be Instantiated.",
        );

        let commands = command_database.get_commands();

        let (event_sender, event_receiver) = unbounded::<Event>();

        Self {
            command_browser: Arc::new(Mutex::new(CommandBrowser::new(event_sender.clone(), commands))),
            add_new_command: Arc::new(Mutex::new(AddNewCommand::new(event_sender.clone()))),
            keybind_help: Arc::new(Mutex::new(KeybindHelp::new(event_sender.clone()))),
            current_activity: None,
            tick_rate: Arc::new(AtomicU64::new(0u64)),
            kill_switch: Arc::new(AtomicBool::new(false)),
            event_receiver,
            event_sender,
            join_handles: Vec::new(),
            command_database,
        }
    }

    pub fn run(&mut self, tick_rate: u64) -> ah::Result<()> {
        self.tick_rate.store(tick_rate, SeqCst);

        let stdout = stdout().into_raw_mode()?;
        let stdout = stdout.into_alternate_screen().unwrap();
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        {
            let tx = self.event_sender.clone();
            let kill_switch = Arc::clone(&self.kill_switch);

            self.join_handles.push(thread::spawn(move || {
                let tty = termion::get_tty().expect("Failed to get TTY");

                for key in tty.keys().flatten() {
                    if let Key::Char('q') = key {
                        kill_switch.store(true, SeqCst);
                        break;
                    }

                    if let Err(e) = tx.send(Event::KeyPressed(key)) {
                        eprintln!("Failed to send key event: {:?}", e);
                        break;
                    } else if kill_switch.load(SeqCst) {
                        break;
                    }
                }
            }));
        }

        {
            let tx = self.event_sender.clone();
            let kill_switch = Arc::clone(&self.kill_switch);
            let tick_rate = Arc::clone(&self.tick_rate);

            self.join_handles.push(thread::spawn(move || loop {
                if let Err(e) = tx.send(Event::Tick) {
                    eprintln!("Failed to send tick event: {:?}", e);
                } else if kill_switch.load(SeqCst) {
                    break;
                }

                thread::sleep(Duration::from_millis(tick_rate.load(SeqCst)));
            }));
        }

        // Set the initial activity the command browser.
        self.current_activity = Some(Activities::CommandBrowser(Arc::clone(
            &self.command_browser,
        )));

        while !self.kill_switch.load(SeqCst) {
            // Draw the current activity.
            if let Some(activity) = &mut self.current_activity {
                activity.draw(&mut terminal);
            } else {
                exit(0);
            }

            if let Ok(event) = self.event_receiver.recv_timeout(Duration::from_millis(100)) {
                match event {
                    Event::KeyPressed(key) => {
                        if let Some(activity) = &mut self.current_activity {
                            activity.on_key_press(key);
                        }
                    }
                    Event::DatabaseUpdate(command) => {
                        self.command_database.update(command);
                    }
                    Event::DatabaseDelete(command) => {
                        self.command_database.delete(command);
                    }
                    _ => continue,
                }
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.kill_switch.store(true, SeqCst);

        for handle in self.join_handles.drain(..) {
            if let Err(e) = handle.join() {
                eprintln!("Failed to join thread: {:?}", e);
                exit(1);
            }
        }
    }
}
