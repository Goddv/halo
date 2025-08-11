// src/ui.rs

use crate::command::CommandLog;
use crate::state::{State, Theme};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

// Colors are now taken from state's theme

pub fn draw(frame: &mut Frame, state: &mut State) {
    let theme = &state.theme;
    frame.render_widget(Block::new().bg(theme.bg), frame.area());

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
    let theme = &state.theme;
    let output_block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(theme.comment))
        .title(Span::styled(
            " [[[ CONSOLE LOG ]]] ",
            Style::new().fg(theme.primary).add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(output_block, area);

    let inner_area = area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut current_y = inner_area.height;

    // Determine which log entry should be highlighted and where to end rendering (scrolling)
    let total_logs = state.command_log.len();
    let active_log_index = if state.scroll_offset > 0 {
        Some(
            total_logs
                .saturating_sub(1)
                .saturating_sub(state.scroll_offset),
        )
    } else {
        None
    };

    // Implement real scrolling: start from an end index based on scroll_offset and render upwards.
    let mut i_opt = total_logs
        .checked_sub(1)
        .map(|last| last.saturating_sub(state.scroll_offset));
    while let Some(i) = i_opt {
        let log = &state.command_log[i];
        let mut block_lines = build_log_block(log, &state.theme);
        let block_height = block_lines.len() as u16;

        // Highlight the active preview block if it matches our calculated index.
        if let Some(active_idx) = active_log_index
            && i == active_idx
        {
            for line in &mut block_lines {
                for span in &mut line.spans {
                    span.style = span.style.fg(theme.accent);
                }
            }
        }

        if block_height <= current_y {
            // Render full block
            current_y = current_y.saturating_sub(block_height);
            let block_area = Rect::new(
                inner_area.x,
                inner_area.y + current_y,
                inner_area.width,
                block_height,
            );
            let paragraph = Paragraph::new(block_lines).wrap(Wrap { trim: false });
            frame.render_widget(paragraph, block_area);
        } else {
            // Render only the bottom part of the block that fits the remaining space.
            let visible_height = current_y;
            if visible_height == 0 {
                break;
            }
            let total_lines = block_lines.len();
            let start_index = total_lines.saturating_sub(visible_height as usize);
            let visible_lines: Vec<Line> = block_lines[start_index..].to_vec();
            let block_area =
                Rect::new(inner_area.x, inner_area.y, inner_area.width, visible_height);
            let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
            frame.render_widget(paragraph, block_area);
            break;
        }

        if i == 0 {
            break;
        }
        i_opt = Some(i - 1);
    }
    // Draw a minimal scrollbar track on the right if there are logs
    if total_logs > 0 {
        let track_x = area.right().saturating_sub(1);
        let track_area = Rect::new(track_x, inner_area.y, 1, inner_area.height);
        // Compute thumb size relative to number of blocks (simple heuristic)
        let min_thumb = 1u16;
        let thumb_h = (inner_area.height / 4).max(min_thumb);
        let max_scroll = total_logs.saturating_sub(1) as u16;
        let scroll = state.scroll_offset.min(max_scroll as usize) as u16;
        let top_space = if max_scroll == 0 {
            0
        } else {
            (inner_area.height - thumb_h) * scroll / max_scroll.max(1)
        };
        let thumb_y = inner_area.y + top_space;
        // track
        frame.render_widget(Block::new().bg(theme.bg), track_area);
        // thumb
        let thumb_char = state.ui.scrollbar_thumb.as_str();
        let thumb = Paragraph::new(Span::styled(thumb_char, Style::new().fg(theme.primary)));
        for y in 0..thumb_h {
            let cell = Rect::new(track_x, thumb_y + y, 1, 1);
            frame.render_widget(thumb.clone(), cell);
        }
    }
}

fn build_log_block<'a>(log: &'a CommandLog, theme: &'a Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let is_empty_prompt = log.command.is_empty() && log.output.is_empty();

    if is_empty_prompt && !log.is_running {
        lines.push(Line::from(vec![
            Span::styled("╭───", Style::new().fg(theme.comment)),
            Span::styled("❯", Style::new().fg(theme.primary)),
        ]));
        lines.push(Line::raw(""));
        return lines;
    }

    let cwd_str = log.cwd.display().to_string();
    lines.push(Line::from(vec![
        Span::styled("╭───", Style::new().fg(theme.comment)),
        Span::styled("❯ ", Style::new().fg(theme.accent)),
        Span::styled(
            &log.command,
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("(", Style::new().fg(theme.comment)),
        Span::styled(
            cwd_str,
            Style::new().fg(theme.comment).add_modifier(Modifier::DIM),
        ),
        Span::styled(")", Style::new().fg(theme.comment)),
    ]));

    if !log.output.is_empty() {
        for output_line in log.output.lines() {
            let content = if let Some(stderr) = output_line.strip_prefix("[stderr] ") {
                Span::styled(
                    stderr,
                    Style::new().fg(theme.error).add_modifier(Modifier::ITALIC),
                )
            } else {
                Span::raw(output_line).fg(theme.fg)
            };
            lines.push(Line::from(vec![
                Span::styled("│  ", Style::new().fg(theme.comment)),
                content,
            ]));
        }
    }

    if log.is_running {
        lines.push(Line::from(vec![
            Span::styled("│  ", Style::new().fg(theme.comment)),
            Span::styled(
                "⚙️  Running...",
                Style::new()
                    .fg(theme.warn)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));
    } else if log.exit_code.is_some() || log.duration_ms.is_some() {
        let code_text = log
            .exit_code
            .map(|c| format!("exit={}", c))
            .unwrap_or_else(|| "exit=?".into());
        let dur_text = log
            .duration_ms
            .map(|d| format!("time={}ms", d))
            .unwrap_or_default();
        let mut meta = vec![
            Span::styled("│  ", Style::new().fg(theme.comment)),
            Span::styled("⏱ ", Style::new().fg(theme.comment)),
            Span::styled(
                code_text,
                if log.exit_code == Some(0) {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(theme.error)
                },
            ),
        ];
        if !dur_text.is_empty() {
            meta.push(Span::raw("  "));
            meta.push(Span::styled(dur_text, Style::new().fg(theme.fg)));
        }
        lines.push(Line::from(meta));
    }

    lines.push(Line::from(Span::styled(
        "╰─",
        Style::new().fg(theme.comment),
    )));
    lines.push(Line::raw(""));

    lines
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &State) {
    let status_layout =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    let theme = &state.theme;
    let version = env!("CARGO_PKG_VERSION");
    let git = state
        .git_branch
        .as_deref()
        .map(|b| format!(" on  {}", b))
        .unwrap_or_default();
    let brand = Paragraph::new(Line::from(vec![
        Span::styled(
            " HALO ",
            Style::new()
                .fg(theme.bg)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{}{} ", version, git),
            Style::new().fg(theme.accent),
        ),
    ]))
    .alignment(Alignment::Left);
    let total_logs = state.command_log.len();
    let pos = if state.scroll_offset > 0 {
        total_logs
            .saturating_sub(1)
            .saturating_sub(state.scroll_offset)
            .saturating_add(1)
    } else {
        total_logs
    };
    let right_text = Line::from(vec![
        Span::styled("📁 ", Style::new().fg(theme.accent)),
        Span::styled(state.cwd.display().to_string(), Style::new().fg(theme.accent)),
        Span::raw("  |  "),
        Span::styled("📄 ", Style::new().fg(theme.accent)),
        Span::styled(format!("{}/{} ", pos, total_logs), Style::new().fg(theme.accent)),
    ]);
    let cwd = Paragraph::new(right_text).alignment(Alignment::Right);
    frame.render_widget(brand, status_layout[0]);
    frame.render_widget(cwd, status_layout[1]);
}

fn render_input_box(frame: &mut Frame, area: Rect, state: &State) {
    let is_previewing = state.scroll_offset > 0;

    let theme = &state.theme;
    let (text, style, border_style, title_span) = if is_previewing {
        // The command to preview is at (len - 1 - scroll_offset), saturating at 0.
        let log_index = state
            .command_log
            .len()
            .saturating_sub(1)
            .saturating_sub(state.scroll_offset);
        let command_text = state
            .command_log
            .get(log_index)
            .map_or("", |log| &log.command);

        (
            Line::from(vec![
                Span::styled(format!("{}  ", state.ui.prompt), Style::new()),
                Span::styled(command_text, Style::new()),
            ]),
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            Style::new().fg(theme.accent),
            {
                const DECOR: &str = "────────────";
                Line::from(vec![
                    Span::styled(DECOR, Style::new().fg(theme.accent)),
                    Span::styled(
                        "[[[ HISTORY PREVIEW ]]]",
                        Style::new().fg(theme.primary).add_modifier(Modifier::BOLD),
                    ),
                ])
            },
        )
    } else {
        (
            Line::from(vec![
                Span::styled(
                    format!("{}  ", state.ui.prompt),
                    Style::new().fg(theme.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&state.input_buffer, Style::new().fg(theme.fg)),
            ]),
            Style::default(),
            Style::new().fg(theme.primary),
            {
                const DECOR: &str = "────────────";
                Line::from(vec![
                    Span::styled(DECOR, Style::new().fg(theme.primary)),
                    Span::styled(
                        format!("[ {} ]", state.username),
                        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ),
                ])
            },
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
    let theme = &state.theme;
    let list = List::new(items)
        .block(
            Block::new()
                .title("💡 Suggestions")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(theme.warn)),
        )
        .highlight_style(
            Style::new()
                .bg(theme.primary)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state =
        ListState::default().with_selected(Some(state.completion_state.selected_index));
    frame.render_widget(Clear, popup_area);
    frame.render_stateful_widget(list, popup_area, &mut list_state);
}
