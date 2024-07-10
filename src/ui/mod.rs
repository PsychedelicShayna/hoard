mod event;
mod search;

use clap::Parser;
use eyre::Result;
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use crate::config::HoardConfig;
use crate::core::trove::Trove;
use crate::core::HoardCmd;
use crate::ui::event::{Config, Event, Events};
use crate::ui::search::controls::{draw_search_key_handler, next_index, previous_index};
use crate::ui::search::render::draw_search_screen;

const DEFAULT_COLLECTIONS: [&str; 2] = ["All", "Local"];

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DrawState {
    Search,
    Explore,
    About,
}

#[derive(Debug, Clone)]
pub struct App {
    /// If set to true, the UI will exit on the next loop iteration
    pub should_exit: bool,
    // ratatui list of commands to display
    pub commands: ListState,
    // vertical scroll position of the command list
    pub vertical_scroll: usize,
    // ratatui list of collections to display
    pub collections: ListState,
    // current screen to draw
    pub screen: DrawState,
    // search string to filter commands displayed at the bottom
    pub search_string: String,
    pub collection: String,

    // Temporary trove that actually gets displayed in the UI
    // This is used to filter the base trove based on search string
    pub search_trove: Trove,

    // Base trove that is used to filter the search_trove
    pub base_trove: Trove,

    pub frame_size: Rect,
}

impl Default for App {
    fn default() -> Self {
        let mut state = Self {
            should_exit: false,
            commands: ListState::default(),
            collections: ListState::default(),
            screen: DrawState::Search,
            search_trove: Trove::default(),
            base_trove: Trove::default(),
            search_string: String::new(),
            collection: String::from(DEFAULT_COLLECTIONS[0]),
            vertical_scroll: 0,
            frame_size: Rect::default(),
        };
        state.commands.select(Some(0));
        state.collections.select(Some(0));
        state
    }
}

impl App {
    pub fn with_trove(&mut self, trove: Trove) -> Self {
        Self {
            base_trove: trove.clone(),
            search_trove: trove,
            ..self.clone()
        }
    }

    pub fn increment_selected_command(&mut self) {
        if self.search_trove.commands.is_empty() {
            return;
        }

        let current_selected = self.commands.selected().unwrap_or(0);
        let next_selected = next_index(current_selected, self.search_trove.commands.len());
        self.commands.select(Some(next_selected));
        // Update the scroll state based on how close the selected command is to the top or bottom
        let actual_position = current_selected - self.vertical_scroll;

        // If we increment at the last element, reset the scroll to 0
        if next_selected == 0 {
            self.vertical_scroll = 0;
        } else if actual_position >= self.frame_size.height as usize - 6 {
            self.vertical_scroll = self.vertical_scroll + 1;
        }
    }

    pub fn decrement_selected_command(&mut self) {
        if self.search_trove.commands.is_empty() {
            return;
        }
        if let Some(selected) = self.commands.selected() {
            let new_selected = previous_index(selected, self.search_trove.commands.len());
            self.commands.select(Some(new_selected));
            // If we jump to the end, just scroll all the way to the bottom
            if new_selected == self.search_trove.commands.len() - 1 {
                self.vertical_scroll = self.search_trove.commands.len().saturating_sub(1);
                return;
            }
        }

        // Update the scroll state based on how close the selected command is to the top or bottom
        let selected = self.commands.selected().unwrap_or(0);
        let max_scroll = self.search_trove.commands.len().saturating_sub(1);
        if selected < self.vertical_scroll {
            self.vertical_scroll = selected;
        } else if selected > self.vertical_scroll + max_scroll {
            self.vertical_scroll = selected.saturating_sub(max_scroll);
        }
    }

    pub fn filter_trove(&mut self) {
        let search_string = self.search_string.to_lowercase();

        if !search_string.is_empty() {
            let filtered: Vec<HoardCmd> = self
                .base_trove
                .commands
                .iter()
                .filter(|cmd| cmd.name.to_lowercase().contains(&search_string))
                .cloned()
                .collect();
            self.search_trove = Trove::from_commands(&filtered);
        } else {
            self.search_trove = self.base_trove.clone();
        }

        self.clip_commands_selection();
    }

    /// Clips the commands selected index to the highest possible index
    /// of the current search_trove commands len
    pub fn clip_commands_selection(&mut self) {
        let selected = self.commands.selected().unwrap_or(0);
        let max = self.search_trove.commands.len().saturating_sub(1);
        if selected > max {
            self.commands.select(Some(max));
        }
    }

    /// Get the selected hoard command from the current search_trove
    pub fn get_selected_hoard_command(&self) -> Option<HoardCmd> {
        self.commands.selected().and_then(|index| {
            if index < self.search_trove.commands.len() {
                Some(self.search_trove.commands[index].clone())
            } else {
                None
            }
        })
    }
}

/// The main entry point for the UI
/// Handles setting up the UI, running the main loop
/// and switching between different screens based on events it recieves
pub fn run(trove: &mut Trove, config: &HoardConfig) -> Result<()> {
    // Setup terminal
    let stdout = stdout().into_raw_mode()?;
    let stdout = stdout.into_alternate_screen().unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // create app and run it
    let tick_rate = Duration::from_millis(200);
    let app = App::default().with_trove(trove.clone());
    let res = run_app(&mut terminal, app, tick_rate);

    // Cleanup
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err}");
        return Err(err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<Option<HoardCmd>> {
    let mut last_tick = Instant::now();
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(tick_rate.as_millis() as u64),
    });

    // Main loop of the UI
    // first match the current DrawState, and draw correct screen
    // then check for any events that might have happened and handle them
    loop {
        let screen = match app.screen {
            DrawState::Search => draw_search_screen,
            DrawState::Explore => not_implemented_ui,
            DrawState::About => not_implemented_ui,
        };

        terminal.draw(|f| screen(f, &mut app))?;

        if let Event::Input(input) = events.next()? {
            match app.screen {
                DrawState::Search => {
                    if let Some(cmd) = draw_search_key_handler(input, &mut app) {
                        return Ok(Some(cmd));
                    }
                }
                DrawState::Explore => {
                    if let Some(cmd) = not_implemented_key_handler(input, &mut app) {
                        return Ok(Some(cmd));
                    }
                }
                DrawState::About => {
                    if let Some(cmd) = not_implemented_key_handler(input, &mut app) {
                        return Ok(Some(cmd));
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_exit {
            terminal.show_cursor()?;
            return Ok(None);
        }
    }
}

pub fn not_implemented_key_handler(input: Key, app: &mut App) -> Option<HoardCmd> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        _ => None,
    }
}

fn not_implemented_ui(frame: &mut Frame, _app: &mut App) {
    frame.render_widget(
        Paragraph::new("Not implemented :(")
            .dark_gray()
            .alignment(Alignment::Center),
        frame.size(),
    );
}

fn partial_highlighted_line<'a>(
    text_input: &'a str,
    search: &'a str,
    highlighted_bg: bool,
) -> Line<'a> {
    // find the index of the search string in the text
    let index = text_input.find(&search);

    match index {
        Some(i) => {
            let (left, right) = text_input.split_at(i);
            let (highlight, rest) = right.split_at(search.len());
            let left_span = Span::raw(left).bg(if highlighted_bg {
                Color::LightBlue
            } else {
                Color::Reset
            });
            let highlight_span = Span::styled(
                highlight,
                Style::default().fg(Color::LightRed).bg(if highlighted_bg {
                    Color::LightBlue
                } else {
                    Color::Reset
                }),
            );
            let rest_span = Span::raw(rest).bg(if highlighted_bg {
                Color::LightBlue
            } else {
                Color::Reset
            });
            Line::from(vec![left_span, highlight_span, rest_span])
        }
        None => Line::from(text_input),
    }
}
