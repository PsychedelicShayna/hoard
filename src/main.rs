// let (command, is_autocomplete) =
// Hoard::default().with_config(None).load_trove().start();

use tui::event_loop::EventLoop;

pub mod cfg;
pub mod cli;
pub mod dbg;
pub mod tui;
pub mod data;
const TICK_RATE: u64 = 100;

#[tokio::main]
async fn main() {
    let mut event_loop = EventLoop::new();
    event_loop.run(TICK_RATE);
}
