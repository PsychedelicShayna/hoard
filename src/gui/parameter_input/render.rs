use crate::config::HoardConfig;
use crate::gui::commands_gui::State;
use crate::util::{split_with_delim, string_find_next, translate_number_to_nth};
use ratatui::backend::TermionBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Terminal;
use termion::screen::AlternateScreen;

pub fn draw(
    app_state: &State,
    config: &HoardConfig,
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>,
    >,
) -> Result<(), eyre::Error> {
    terminal.draw(|rect| {
        let size = rect.size();
        // Overlay
        let overlay_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(size);

        let mut query_string = config.query_prefix.clone();
        query_string.push_str(&app_state.input.clone()[..]);

        let title_string = format!(
            "Provide {} parameter",
            translate_number_to_nth(app_state.provided_parameter_count)
        );

        let command_style = Style::default().fg(Color::Rgb(
            config.command_color.unwrap().0,
            config.command_color.unwrap().1,
            config.command_color.unwrap().2,
        ));

        let primary_style = Style::default().fg(Color::Rgb(
            config.primary_color.unwrap().0,
            config.primary_color.unwrap().1,
            config.primary_color.unwrap().2,
        ));

        let input = Paragraph::new(query_string)
            .style(primary_style)
            .block(Block::default().style(command_style).title(title_string));

        let command_text = app_state
            .selected_command
            .as_ref()
            .unwrap()
            .command
            .as_str();

        let token = config.parameter_token.as_ref().unwrap().as_str();
        let ending_token = config.parameter_ending_token.as_ref().unwrap().as_str();

        // Named parameter ending with a space
        let named_token = string_find_next(command_text, token, " ");

        // Named parameter ending with ending token. If ending token is not used, `full_named_token` is an empty string
        let mut full_named_token = string_find_next(command_text, token, ending_token);

        full_named_token.push_str(ending_token);
        // Select the split based on whether the ending token is part of the command or not
        
        let split_token = if command_text.contains(ending_token) {
            full_named_token
        } else {
            named_token
        };

        let mut command_spans: Vec<Span> = Vec::new();
        let split_commands: Vec<String> = split_with_delim(command_text, &split_token);

        if token == split_token {
            // If the next token to replace is not named
            let command_parts = command_text.split_once(token);

            let mut spans: Vec<Span> = if let Some((begin, end)) = command_parts {
                vec![
                    Span::styled(begin, command_style),
                    Span::styled(token, primary_style),
                    Span::styled(end, command_style),
                ]
            } else {
                vec![Span::styled(command_text, command_style)]
            };

            command_spans.append(&mut spans);
        } else {
            // if the next token to replaced is named, find all other occurrences and paint them too
            let mut spans = split_commands
                .iter()
                .map(|e| {
                    if *e == split_token {
                        (e, primary_style)
                    } else {
                        (e, command_style)
                    }
                })
                .map(|(command, style)| Span::styled(command, style))
                .collect();
            command_spans.append(&mut spans);
        }

        let command = Paragraph::new(Line::from(command_spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().style(primary_style));

        rect.render_widget(command, overlay_chunks[1]);
        rect.render_widget(input, overlay_chunks[2]);
    })?;

    Ok(())
}
