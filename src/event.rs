// src/event.rs

use crate::app::App;
use crate::error::AppResult;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub struct EventHandler;

impl EventHandler {
    pub async fn handle_event(&self, event: Event, app: &mut App) -> AppResult<()> {
        app.state.needs_redraw = true;

        match event {
            Event::Key(key_event) => self.handle_key_press(key_event, app).await?,
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, app),
            _ => {}
        }

        Ok(())
    }

    async fn handle_key_press(&self, key: KeyEvent, app: &mut App) -> AppResult<()> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            if app.state.completion_state.active {
                app.state.completion_state.stop_completion();
            } else {
                app.kill_command()?;
            }
            return Ok(());
        }

        if matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace) {
            app.state.exit_preview_mode();
        }

        if app.state.completion_state.active {
            self.handle_completion_mode_key(key, app);
        } else {
            self.handle_normal_mode_key(key, app);
        }
        Ok(())
    }

    fn handle_completion_mode_key(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Tab | KeyCode::Down => app.state.completion_state.next_suggestion(),
            KeyCode::BackTab | KeyCode::Up => app.state.completion_state.previous_suggestion(),
            KeyCode::Enter => {
                if let Some((new_input, new_cursor)) = app
                    .state
                    .completion_state
                    .apply_completion(&app.state.input_buffer)
                {
                    app.state.input_buffer = new_input;
                    app.state.cursor_position = new_cursor;
                }
                app.state.completion_state.stop_completion();
            }
            KeyCode::Esc => app.state.completion_state.stop_completion(),
            _ => {
                app.state.completion_state.stop_completion();
                self.handle_normal_mode_key(key, app);
            }
        }
    }

    fn handle_normal_mode_key(&self, key: KeyEvent, app: &mut App) {
        // CORRECTED: max_scroll allows scrolling to the very first command.
        let max_scroll = app.state.command_log.len();
        match key.code {
            KeyCode::Char(c) => app.state.insert_char(c),
            KeyCode::Backspace => app.state.backspace(),
            KeyCode::Left => app.state.move_cursor_left(),
            KeyCode::Right => app.state.move_cursor_right(),
            KeyCode::Up => self.navigate_history_up(app),
            KeyCode::Down => self.navigate_history_down(app),
            KeyCode::Enter => app.submit_command(),
            KeyCode::Tab => {
                let (input, cwd) = (app.state.input_buffer.clone(), app.state.cwd.clone());
                app.state.completion_state.start_completion(&input, &cwd);
            }
            KeyCode::PageUp => {
                app.state.scroll_offset = (app.state.scroll_offset + 5).min(max_scroll)
            }
            KeyCode::PageDown => {
                app.state.scroll_offset = app.state.scroll_offset.saturating_sub(5)
            }
            _ => {}
        }
    }

    fn handle_mouse_event(&self, mouse: MouseEvent, app: &mut App) {
        // CORRECTED: max_scroll allows scrolling to the very first command.
        let max_scroll = app.state.command_log.len();
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                app.state.scroll_offset = (app.state.scroll_offset + 1).min(max_scroll);
            }
            MouseEventKind::ScrollDown => {
                app.state.scroll_offset = app.state.scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn navigate_history_up(&self, app: &mut App) {
        if app.state.scroll_offset > 0 {
            return;
        }
        if app.state.history.is_empty() {
            return;
        }
        let new_index = match app.state.history_index {
            Some(idx) => idx.saturating_sub(1),
            None => app.state.history.len() - 1,
        };
        app.state.history_index = Some(new_index);
        app.state.input_buffer = app.state.history[new_index].clone();
        app.state.cursor_position = app.state.input_buffer.len();
    }

    fn navigate_history_down(&self, app: &mut App) {
        if app.state.scroll_offset > 0 {
            return;
        }
        if app.state.history.is_empty() {
            return;
        }
        match app.state.history_index {
            Some(idx) if idx < app.state.history.len() - 1 => {
                let new_index = idx + 1;
                app.state.history_index = Some(new_index);
                app.state.input_buffer = app.state.history[new_index].clone();
                app.state.cursor_position = app.state.input_buffer.len();
            }
            _ => {
                app.state.history_index = None;
                app.state.input_buffer.clear();
                app.state.cursor_position = 0;
            }
        }
    }
}
