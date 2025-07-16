// src/ui.rs

use crate::command::CommandLog;
use crate::state::State;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

// --- Theme Colors ---
const COLOR_PRIMARY: Color = Color::Rgb(0, 184, 255); // Vibrant Cyan
const COLOR_ACCENT: Color = Color::Rgb(255, 6, 119); // Vibrant Magenta
const COLOR_WARN: Color = Color::Rgb(255, 255, 0); // Bright Yellow
const COLOR_ERROR: Color = Color::Rgb(255, 85, 85); // Bright Red
const COLOR_FG: Color = Color::Rgb(229, 233, 240); // Light Gray (almost white)
const COLOR_BG: Color = Color::Rgb(22, 24, 33); // Very Dark Blue
const COLOR_COMMENT: Color = Color::Rgb(76, 86, 106); // Grayish Blue
const COLOR_DIM: Color = Color::Rgb(50, 56, 70); // Dimmed Blue/Gray

pub fn draw(frame: &mut Frame, state: &mut State) {
    frame.render_widget(Block::new().bg(COLOR_BG), frame.area());

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_output_log(frame, main_layout[0], state);
    render_status_bar(frame, main_layout[1], state);
    render_input_box(frame, main_layout[2], state);

    if state.completion_state.active {
        render_completion_popup(frame, main_layout[2], state);
    }

    if state.scroll_offset == 0 {
        let input_block = Block::default().borders(Borders::ALL);
        let inner_area = input_block.inner(main_layout[2]);
        let prompt_width = 3;

        frame.set_cursor_position((
            inner_area.x + prompt_width + state.cursor_position as u16,
            inner_area.y,
        ));
    }
}

fn render_output_log(frame: &mut Frame, area: Rect, state: &State) {
    let output_block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(COLOR_COMMENT))
        .title(Span::styled(
            " [[[ CONSOLE LOG ]]] ",
            Style::new().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(output_block, area);

    let inner_area = area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut current_y = inner_area.height;

    // Determine which log entry should be highlighted
    let active_log_index = if state.scroll_offset > 0 {
        Some(state.command_log.len().saturating_sub(state.scroll_offset))
    } else {
        None
    };

    // Always iterate through all logs in reverse. We don't skip anymore.
    // The scrolling is achieved by only rendering what fits.
    for (i, log) in state.command_log.iter().enumerate().rev() {
        let mut block_lines = build_log_block(log);
        let block_height = block_lines.len() as u16;

        if current_y < block_height {
            break;
        }

        current_y = current_y.saturating_sub(block_height);
        let block_area = Rect::new(
            inner_area.x,
            inner_area.y + current_y,
            inner_area.width,
            block_height,
        );

        // Highlight the active preview block if it matches our calculated index.
        if let Some(active_idx) = active_log_index
            && i == active_idx
        {
            for line in &mut block_lines {
                for span in &mut line.spans {
                    span.style = span.style.fg(COLOR_ACCENT);
                }
            }
        }

        let paragraph = Paragraph::new(block_lines);
        frame.render_widget(paragraph, block_area);
    }
}

fn build_log_block(log: &CommandLog) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let is_empty_prompt = log.command.is_empty() && log.output.is_empty();

    if is_empty_prompt && !log.is_running {
        lines.push(Line::from(vec![
            Span::styled("╭─ ", Style::new().fg(COLOR_COMMENT)),
            Span::styled("❯", Style::new().fg(COLOR_PRIMARY)),
        ]));
        lines.push(Line::raw(""));
        return lines;
    }

    lines.push(Line::from(vec![
        Span::styled("╭─ ", Style::new().fg(COLOR_COMMENT)),
        Span::styled("❯ ", Style::new().fg(COLOR_ACCENT)),
        Span::styled(
            &log.command,
            Style::new().fg(COLOR_FG).add_modifier(Modifier::BOLD),
        ),
    ]));

    if !log.output.is_empty() {
        for output_line in log.output.lines() {
            let content = if let Some(stderr) = output_line.strip_prefix("[stderr] ") {
                Span::styled(
                    stderr,
                    Style::new().fg(COLOR_ERROR).add_modifier(Modifier::ITALIC),
                )
            } else {
                Span::raw(output_line).fg(COLOR_FG)
            };
            lines.push(Line::from(vec![
                Span::styled("│  ", Style::new().fg(COLOR_COMMENT)),
                content,
            ]));
        }
    }

    if log.is_running {
        lines.push(Line::from(vec![
            Span::styled("│  ", Style::new().fg(COLOR_COMMENT)),
            Span::styled(
                "⚙️  Running...",
                Style::new()
                    .fg(COLOR_WARN)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));
    }

    lines.push(Line::from(Span::styled(
        "╰─",
        Style::new().fg(COLOR_COMMENT),
    )));
    lines.push(Line::raw(""));

    lines
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &State) {
    let status_layout =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    let brand = Paragraph::new(Line::from(vec![
        Span::styled(
            "[[[ HALO ]]]",
            Style::new()
                .fg(COLOR_BG)
                .bg(COLOR_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v0.1.0 ", Style::new().fg(COLOR_ACCENT)),
    ]))
    .alignment(Alignment::Left);
    let cwd_display = format!("📁 {} ", state.cwd.display());
    let cwd = Paragraph::new(Span::styled(cwd_display, Style::new())).alignment(Alignment::Right);
    frame.render_widget(brand, status_layout[0]);
    frame.render_widget(cwd, status_layout[1]);
}

fn render_input_box(frame: &mut Frame, area: Rect, state: &State) {
    let is_previewing = state.scroll_offset > 0;

    let (text, style, border_style, title_span) = if is_previewing {
        // The command to preview is now at this robust index.
        let log_index = state.command_log.len().saturating_sub(state.scroll_offset);
        let command_text = state
            .command_log
            .get(log_index)
            .map_or("", |log| &log.command);

        (
            Line::from(vec![
                Span::styled("❯  ", Style::new()),
                Span::styled(command_text, Style::new()),
            ]),
            Style::new().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD),
            Style::new().fg(COLOR_ACCENT),
            Span::styled(
                " [[[ HISTORY PREVIEW ]]] ",
                Style::new().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD),
            ),
        )
    } else {
        (
            Line::from(vec![
                Span::styled(
                    "❯  ",
                    Style::new().fg(COLOR_PRIMARY).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&state.input_buffer, Style::new().fg(COLOR_FG)),
            ]),
            Style::default(),
            Style::new().fg(COLOR_PRIMARY),
            Span::styled(
                format!("  [[[ {} ]]]  ", state.username),
                Style::new().fg(COLOR_ACCENT).add_modifier(Modifier::BOLD),
            ),
        )
    };

    let input_paragraph = Paragraph::new(text).style(style).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(title_span),
    );

    frame.render_widget(input_paragraph, area);
}

fn render_completion_popup(frame: &mut Frame, area: Rect, state: &mut State) {
    let suggestions = &state.completion_state.suggestions;
    let items: Vec<ListItem> = suggestions
        .iter()
        .map(|s| {
            let icon = if s.ends_with('/') { "📁" } else { "📄" };
            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::raw(" "),
                Span::raw(s),
            ]))
        })
        .collect();
    let height = (items.len() + 2).min(10) as u16;
    let popup_area = Rect {
        x: area.x,
        y: area.y.saturating_sub(height),
        width: area.width.min(80),
        height,
    };
    let list = List::new(items)
        .block(
            Block::new()
                .title("💡 Suggestions")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(COLOR_WARN)),
        )
        .highlight_style(
            Style::new()
                .bg(COLOR_PRIMARY)
                .fg(COLOR_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state =
        ListState::default().with_selected(Some(state.completion_state.selected_index));
    frame.render_widget(Clear, popup_area);
    frame.render_stateful_widget(list, popup_area, &mut list_state);
}
