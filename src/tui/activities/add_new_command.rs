use anyhow as ah;
use crossbeam_channel::Sender;
use termion::event::Key;

use crate::tui::event_loop::Event;

use super::{Activity, TermHandle};

pub struct AddNewCommand {
    event_loop_sender_tx: Sender<Event>,
}

impl AddNewCommand {
    pub fn new(event_loop_sender_tx: Sender<Event>) -> Self {
        Self {
            event_loop_sender_tx,
        }
    }
}

impl Activity for AddNewCommand {
    fn on_key_press(&mut self, key: Key) {
        println!(
            "{}:{} CommandBrowser key pressed: {:?}",
            file!(),
            line!(),
            key
        );
    }
    fn draw(&mut self, terminal: &mut TermHandle) {}
    fn signal_event_loop(&mut self, event: Event) -> ah::Result<()> {
        self.event_loop_sender_tx.send(event)?;
        Ok(())
    }
}
