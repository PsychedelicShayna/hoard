use crate::core::HoardCmd;
use crate::ui::App;

use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn draw_search_key_handler(input: Key, app: &mut App) -> Option<HoardCmd> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::Backspace => {
            app.search_string.pop();
            app.filter_trove();
            None
        }
        Key::Ctrl('w') => {
            // Delete the last word from searchstring
            let mut search_string = app.search_string.split_whitespace();
            search_string.next_back();
            // Collect with spaces
            app.search_string = search_string.collect::<Vec<&str>>().join(" ");
            app.filter_trove();
            None
        }
        Key::Ctrl('u') => {
            // Deletes the entire search string
            app.search_string.clear();
            app.filter_trove();
            None
        }
        Key::Down => { 
            app.increment_selected_command();
            None
        }
        Key::Up => { 
            app.decrement_selected_command();
            None
        }
        Key::Char(c) => {
            app.search_string.push(c);
            app.filter_trove();
            None
        }
        _ => None,
    }
}

pub const fn next_index(current_index: usize, collection_length: usize) -> usize {
    if current_index >= collection_length - 1 {
        0
    } else {
        current_index + 1
    }
}

pub const fn previous_index(current_index: usize, collection_length: usize) -> usize {
    if current_index == 0 {
        collection_length - 1
    } else {
        current_index - 1
    }
}
