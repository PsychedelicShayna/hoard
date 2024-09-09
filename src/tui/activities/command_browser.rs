use crossbeam_channel::Sender;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use termion::event::Key;

use crate::tui::event_loop::Event;
use anyhow as ah;

use super::{Activity, TermHandle};
use crate::data::models::Command;

/// Unicode character to act as a mock cursor for input fields.
const IBEAM: &str = "▕";

#[derive(Clone, Debug)]
pub enum SelectablePanel {
    /// left panel containing the list of binary names of all stored commands.
    CommandList,

    /// Part of the top of the right panel, containing the list of tags for
    /// the selected binary on the left panel.
    CommandTags,

    /// The middle section of the right panel, containing the description of
    /// the selected command.
    CommandDescription,

    /// Bottom section of the right panel, containing a list of different
    /// stored ways to invoke the command with different arguments.
    CommandInvocations,
}

#[derive(Clone, Debug)]
pub enum ViMode {
    /// Navigate between panels, and perform actions through keybinds.
    Normal,

    /// Solely for editing the text of a selected, editable panel.
    /// There are no keybinds during insert mode other than leaving it.
    Insert,
}

impl ViMode {
    pub fn toggle(&mut self) {
        match self {
            ViMode::Normal => *self = ViMode::Insert,
            ViMode::Insert => *self = ViMode::Normal,
        }
    }
}

impl From<ViMode> for String {
    fn from(mode: ViMode) -> String {
        match mode {
            ViMode::Normal => "Normal".to_string(),
            ViMode::Insert => "Insert".to_string(),
        }
    }
}

pub struct CommandBrowser {
    event_loop_sender_tx: Sender<Event>,
    selected_command_index: Option<usize>,
    command_list: Vec<Command>,
    command_list_state: ListState,
    selected_panel: SelectablePanel,
    vimode: ViMode,
    search_filter: String,

    /// When in insert mode, this holds a copy of the text being edited,
    /// and is written back to where the copy was made from upon leaving.
    insert_mode_buffer: String,
}

struct CommandBrowserWidgets<'a> {
    list_widget: List<'a>,
    selected_command_widget: Option<Paragraph<'a>>,
    tags_widget: Paragraph<'a>,
    search_bar_widget: Paragraph<'a>,
    description_widget: Paragraph<'a>,
}

impl CommandBrowser {
    pub fn new(event_loop_sender_tx: Sender<Event>, commands: Vec<Command>) -> Self {
        Self {
            event_loop_sender_tx,
            selected_command_index: None,
            command_list: commands,
            selected_panel: SelectablePanel::CommandList,
            vimode: ViMode::Normal,
            command_list_state: ListState::default(),
            insert_mode_buffer: String::default(),
            search_filter: String::default(),
        }
    }

    /// Generates selectable panels that can be cycled between and edited.
    pub fn generate_panels<'a>(&mut self) -> CommandBrowserWidgets<'a> {
        // The frame of the command list.
        let list_area = Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title(" Commands ")
            .border_type(BorderType::Plain);

        // Maps the internally stored command list into drawable ListItems.
        let items: Vec<ListItem> = self
            .command_list
            .iter()
            .map(|command| {
                ListItem::new(Line::from(vec![Span::styled(
                    command.binary.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        if self
            .selected_command_index
            .is_some_and(|i| i >= self.command_list.len())
            || self.command_list.is_empty()
        {
            self.selected_command_index = None;
        }

        // If somehow the index became an invalid Some value, reset it to None.
        let selected_command: Option<&Command> = self
            .selected_command_index
            .and_then(|i| self.command_list.get(i));

        let list_widget = List::new(items)
            .block(list_area)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let selected_command: Option<&Command> = self
            .selected_command_index
            .and_then(|i| self.command_list.get(i));

        // If no command is selected, there is no item in the list widget for
        // which the paragraph applies to, so it must be an Option.
        let selected_command_widget: Option<Paragraph<'_>> = selected_command.map(|command| {
            Paragraph::new(command.binary.clone())
                .style(Style::default())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .title(" Selected Command ")
                        .border_type(BorderType::Plain),
                )
        });

        let mut tags_str: String = selected_command
            .map(|command| command.tags.join(", "))
            .unwrap_or_default();

        // Only add a cursor to the tags string when in insert mode.
        if let ViMode::Insert = self.vimode {
            tags_str.push_str(IBEAM);
        }

        // Unlike the list widget, the paragraph where the tags are supposed
        // to go can be empty, as it's not part of a list widget.
        let tags_widget: Paragraph<'_> = Paragraph::new(tags_str)
            .style(Style::default())
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .title(" Tags ")
                    .border_type(BorderType::Plain),
            );

        // The same goes for the description paragraph.
        let mut description_str: String = selected_command
            .map(|command| command.description.clone())
            .unwrap_or_default();

        // Only add a cursor to the description string when in insert mode.
        if let ViMode::Insert = self.vimode {
            description_str.push_str(IBEAM);
        }

        let description_widget: Paragraph<'_> = Paragraph::new(description_str)
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .title(" Description ")
                    .border_type(BorderType::Plain),
            );

        let mut search_bar_string = format!(" > {}", self.search_filter);

        // Only add a cursor to the search bar string when in insert mode.
        if let ViMode::Insert = self.vimode {
            search_bar_string.push_str(IBEAM);
        }

        let search_bar_widget = Paragraph::new(search_bar_string).block(
            Block::default()
                .style(Style::default())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

        CommandBrowserWidgets {
            list_widget,
            selected_command_widget,
            tags_widget,
            search_bar_widget,
            description_widget,
        }
    }
}

impl Activity for CommandBrowser {
    fn on_key_press(&self, key: Key) {}

    fn draw(&mut self, terminal: &mut TermHandle) {
        let namespace_tabs = ["All", "Default", "Personal", "Work"];

        terminal.draw(|rect| {
            let size = rect.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(size);

            let menu: Vec<Line> = namespace_tabs
                .iter()
                .map(|t| Line::from(vec![Span::styled(*t, Style::default())]))
                .collect();

            let namespace_widget = Tabs::new(menu)
                .block(
                    Block::default()
                        .title(" Hoard Namespace ")
                        .borders(Borders::ALL),
                )
                .style(Style::default())
                .highlight_style(Style::default().add_modifier(Modifier::UNDERLINED))
                .divider(Span::raw("|"));

            rect.render_widget(namespace_widget, chunks[0]);

            // The layout for the left and right panels.
            let command_list_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[1]);

            // Unsure what this layout is for, yet.
            let command_detail_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(60),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(command_list_layout[1]);

            let panel_widgets = self.generate_panels();
            let list_widget = &panel_widgets.list_widget;
            let tags_widget = &panel_widgets.tags_widget;
            let description = &panel_widgets.description_widget;
            let search_bar = &panel_widgets.search_bar_widget;
            let commands = &panel_widgets.selected_command_widget;

            // Render the list widget.
            rect.render_stateful_widget(
                list_widget,
                command_list_layout[0],
                &mut self.command_list_state,
            );

            // Render the tags widget.
            rect.render_widget(tags_widget, command_detail_layout[0]);

            // Render the description widget.
            rect.render_widget(description, command_detail_layout[1]);

            // Normally we'd render the invocation list here, but we haven't generated
            // that widget yet.

            // Render the search bar widget.
            rect.render_widget(search_bar, chunks[2]);

            let list_state = ListState::default();

            let (footer_constraint_left, footer_constraint_right) = match self.vimode {
                ViMode::Normal => (50, 50),
                ViMode::Insert => (99, 1)
            };

            let footer_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([
                    Constraint::Percentage(footer_constraint_left),
                    Constraint::Percentage(footer_constraint_right),
                ])
                .split(chunks[3]);

            let mode_string: String = self.vimode.clone().into();

            let mode_hint_widget = Paragraph::new(format!("[{}] ", mode_string))
                .alignment(Alignment::Left)
                .style(Style::default());

            let shortcut_hint_widget = Paragraph::new("Press 'i' to enter insert mode.")
                .alignment(Alignment::Right)
                .style(Style::default());

            rect.render_widget(mode_hint_widget, footer_chunk[0]);
            rect.render_widget(shortcut_hint_widget, footer_chunk[1]);
        });
    }

    fn signal_event_loop(&self, event: Event) -> ah::Result<()> {
        self.event_loop_sender_tx.send(event)?;
        Ok(())
    }
}
