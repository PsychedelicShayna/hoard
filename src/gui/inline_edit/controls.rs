use crate::core::{string_to_tags, HoardCmd};
use crate::gui::commands_gui::{ControlState, EditSelection, State, ViMode};
use crate::gui::list_search::controls::{next_index, previous_index, switch_namespace};
use termion::event::Key;

pub fn key_handler(
    input: Key,
    state: &mut State,
    trove_commands: &[HoardCmd],
    namespace_tabs: &[&str],
) -> Option<HoardCmd> {
    match (&state.vimode, input) {
        (ViMode::Insert, Key::Esc) => {
            state.vimode = ViMode::Normal;
            None
        }

        (ViMode::Normal, Key::Char('i')) => {
            state.vimode = ViMode::Insert;
            None
        }

        // Go one namespace to the left.
        (ViMode::Normal, Key::Char('d')) => {
            if let Some(selected) = state.namespace_tab.selected() {
                let new_selected_tab = previous_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }
        // Go one namespace the right
        (ViMode::Normal, Key::Char('u')) => {
            if let Some(selected) = state.namespace_tab.selected() {
                let new_selected_tab = next_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }

        // Only exit the edit mode
        (ViMode::Normal, Key::Char('h')) => {
            state.control = ControlState::Search;
            None
        }

        (ViMode::Insert, Key::Char('\n')) => {
            state.vimode = ViMode::Normal;

            let mut edited_command = state.selected_command.clone().unwrap();
            let new_string = state.string_to_edit.clone();

            match state.edit_selection {
                EditSelection::Description => edited_command.description = new_string,
                EditSelection::Command => edited_command.command = new_string,
                EditSelection::Tags => edited_command.tags = string_to_tags(&new_string),
                EditSelection::Name | EditSelection::Namespace => (),
            };

            Some(edited_command)
        }
        // (ViMode::Normal, Key::Char('\n')) => {
        //     state.vimode = ViMode::Insert;
        //     None
        // },
        (ViMode::Normal, Key::Char('j')) => {
            state.edit_selection = state.edit_selection.next();
            state.update_string_to_edit();
            None
        }
        (ViMode::Normal, Key::Char('k')) => {
            state.edit_selection = state.edit_selection.prev();
            state.update_string_to_edit();
            None
        }
        (ViMode::Normal, Key::Ctrl('c' | 'd' | 'g') | Key::Char('q')) => {
            // Definitely exit program
            state.should_exit = true;
            None
        }
        (ViMode::Insert, Key::Backspace) => {
            state.string_to_edit.pop();
            None
        }
        (ViMode::Insert, Key::Char(c)) => {
            state.string_to_edit.push(c);
            None
        }
        _ => None,
    }
}
