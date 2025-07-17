// src/state.rs

use crate::command::CommandLog;
use crate::completion::CompletionState;
use crate::error::AppResult;
use std::path::PathBuf;

const HISTORY_LIMIT: usize = 100;

pub struct State {
    pub should_quit: bool,
    pub needs_redraw: bool,
    pub username: String,
    pub cwd: PathBuf,
    pub git_branch: Option<String>, // Added to store git branch info
    pub input_buffer: String,
    pub cursor_position: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub command_log: Vec<CommandLog>,
    pub scroll_offset: usize,
    pub completion_state: CompletionState,
}

impl State {
    pub fn new() -> AppResult<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self {
            should_quit: false,
            needs_redraw: true,
            username: users::get_current_username()
                .and_then(|name| name.into_string().ok())
                .unwrap_or_else(|| "user".to_string()),
            cwd: cwd.clone(),
            git_branch: None, // Will be updated by the app loop
            input_buffer: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
            command_log: vec![CommandLog::new(
                "".into(),
                "Welcome to Halo! A modern shell for a modern age.".into(),
                false,
                cwd, // Store the CWD with the welcome message
            )],
            scroll_offset: 0,
            completion_state: CompletionState::new(),
        })
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor_position = self.cursor_position.min(self.input_buffer.len());
    }

    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn exit_preview_mode(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn add_log_entry(&mut self, command: String, cwd: PathBuf) {
        self.command_log
            .push(CommandLog::new(command, String::new(), true, cwd));
        if self.command_log.len() > HISTORY_LIMIT {
            self.command_log.remove(0);
        }
    }

    pub fn append_to_last_log(&mut self, line: String) {
        if let Some(last) = self.command_log.last_mut() {
            if !last.output.is_empty() {
                last.output.push('\n');
            }
            last.output.push_str(&line);
            self.needs_redraw = true;
        }
    }

    pub fn finish_last_log(&mut self) {
        if let Some(last) = self.command_log.last_mut() {
            last.is_running = false;
            self.needs_redraw = true;
        }
    }
}
