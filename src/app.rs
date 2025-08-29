// src/app.rs

use crate::command::{CommandLog, CommandManager, CommandUpdate};
use crate::error::AppResult;
use crate::event::EventHandler;
use crate::state::State;
use crate::ui;
use ratatui::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub struct App {
    pub state: State,
    command_manager: CommandManager,
    command_update_rx: UnboundedReceiver<CommandUpdate>,
    command_update_tx: UnboundedSender<CommandUpdate>,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            state: State::new()?,
            command_manager: CommandManager::new(),
            command_update_rx: rx,
            command_update_tx: tx,
        })
    }

    /// Fetches git info and updates the state.
    fn update_git_info(&mut self) {
        self.state.git_branch = get_git_branch(&self.state.cwd);
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> AppResult<()> {
        let event_handler = EventHandler;

        while !self.state.should_quit {
            self.process_command_updates();
            self.update_git_info();

            if self.state.needs_redraw {
                terminal.draw(|frame| {
                    ui::draw(frame, &mut self.state);
                })?;
                self.state.needs_redraw = false;
            }

            if crossterm::event::poll(Duration::from_millis(100))? {
                let event = crossterm::event::read()?;
                event_handler.handle_event(event, self).await?;
            }
        }
        Ok(())
    }

    pub fn submit_command(&mut self) {
        let input = self.state.input_buffer.trim().to_string();
        self.state.exit_preview_mode();

        let current_cwd = self.state.cwd.clone();

        if input.is_empty() {
            let last_was_empty = self
                .state
                .command_log
                .last()
                .is_some_and(|l| l.command.is_empty() && l.output.is_empty());
            if !last_was_empty {
                self.state.command_log.push(CommandLog::new(
                    "".into(),
                    "".into(),
                    false,
                    current_cwd,
                ));
            }
            return;
        }

        self.state.add_log_entry(input.clone(), current_cwd);
        if self.state.history.last() != Some(&input) {
            self.state.history.push(input.clone());
            if let Err(e) = self.state.save_history() {
                self.state
                    .append_to_last_log(format!("[history save error] {e}"));
            }
        }

        self.state.input_buffer.clear();
        self.state.cursor_position = 0;

        let parts = match shlex::split(&input) {
            Some(parts) if !parts.is_empty() => parts,
            _ => {
                self.state
                    .append_to_last_log("Error: Mismatched quotes.".into());
                self.state.finish_last_log();
                return;
            }
        };

        let mut cmd = parts[0].clone();
        let mut args: Vec<String> = parts[1..].to_vec();

        match cmd.as_str() {
            "exit" => self.state.should_quit = true,
            ":reload" => {
                self.state.load_config();
                self.state.append_to_last_log("[config reloaded]".into());
            }
            "theme" => {
                if args.is_empty() {
                    self.state
                        .append_to_last_log(format!("theme: {}", self.state.theme_name.clone()));
                } else if args.get(0).map(|s| s.as_str()) == Some("set") {
                    if let Some(name) = args.get(1) {
                        if self.state.load_theme_from_file(name) {
                            let _ = self.state.save_session();
                            self.state.append_to_last_log(format!("[theme set to {}]", name));
                        } else {
                            self.state.append_to_last_log(format!("[error: theme '{}' not found]", name));
                        }
                    } else {
                        // Enter interactive theme selection mode
                        self.state.enter_theme_selection_mode();
                        self.state.append_to_last_log("Theme selection mode - use ↑/↓ to navigate, Enter to select, Esc to cancel".into());
                    }
                } else if args.get(0).map(|s| s.as_str()) == Some("list") {
                    let themes = self.state.get_available_themes();
                    self.state.append_to_last_log("Available themes:".into());
                    for theme in themes {
                        self.state.append_to_last_log(format!("  {}", theme));
                    }
                } else if args.get(0).map(|s| s.as_str()) == Some("refresh") {
                    if let Err(e) = crate::themes::refresh_themes() {
                        self.state.append_to_last_log(format!("[error: failed to refresh themes: {}]", e));
                    } else {
                        self.state.append_to_last_log("[themes refreshed successfully]".into());
                    }
                } else {
                    self.state.append_to_last_log("usage: theme [set <name> | list | refresh]".into());
                }
            }
            "alias" => {
                if args.is_empty() {
                    if self.state.aliases.is_empty() {
                        self.state.append_to_last_log("(no aliases)".into());
                    } else {
                        let mut pairs: Vec<(String, String)> = self
                            .state
                            .aliases
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        pairs.sort_by(|a, b| a.0.cmp(&b.0));
                        for (k, v) in pairs {
                            self.state.append_to_last_log(format!("alias {k}='{v}'"));
                        }
                    }
                } else {
                    self.state
                        .append_to_last_log("usage: alias  # lists aliases".into());
                }
            }
            "cd" => self.handle_cd(&args),
            "pwd" => self
                .state
                .append_to_last_log(self.state.cwd.display().to_string()),
            _ => {
                // Minimal alias expansion (from halo.toml)
                if let Some(expanded) = self.state.aliases.get(&cmd) {
                    let rest = if args.is_empty() {
                        String::new()
                    } else {
                        args.join(" ")
                    };
                    let combined = if rest.is_empty() {
                        expanded.clone()
                    } else {
                        format!("{expanded} {rest}")
                    };
                    if let Some(new_parts) = shlex::split(&combined) {
                        if !new_parts.is_empty() {
                            cmd = new_parts[0].clone();
                            args = new_parts[1..].to_vec();
                        }
                    }
                }

                // track start time for duration
                self.state.mark_last_log_started();
                if let Err(e) = self.command_manager.spawn_command(
                    &cmd,
                    &args,
                    &self.state.cwd,
                    self.command_update_tx.clone(),
                ) {
                    self.state.append_to_last_log(format!("{cmd}: {e}"));
                    self.state.finish_last_log();
                }
                return;
            }
        }
        self.state.finish_last_log();
    }

    fn handle_cd(&mut self, args: &[String]) {
        let target = args.first().map_or("~", |s| s.as_str());
        let new_dir = expand_cd_target(target, &self.state.cwd);

        if let Err(e) = std::env::set_current_dir(&new_dir) {
            self.state.append_to_last_log(format!("cd: {e}"));
        } else if let Ok(cwd) = std::env::current_dir() {
            self.state.cwd = cwd;
            let _ = self.state.save_session();
        }
    }

    pub fn kill_command(&mut self) -> AppResult<()> {
        self.command_manager.kill_running_command()?;
        self.state
            .append_to_last_log("[Process killed by user]".into());
        Ok(())
    }

    fn process_command_updates(&mut self) {
        while let Ok(update) = self.command_update_rx.try_recv() {
            match update {
                CommandUpdate::NewLine(line) => self.state.append_to_last_log(line),
                CommandUpdate::Finished(code) => self.state.finish_last_log_with_result(code),
            }
            self.state.needs_redraw = true;
        }
    }
}

// Helper to get the git branch, returning a clean string for the UI.
fn get_git_branch(path: &Path) -> Option<String> {
    let repo = git2::Repository::discover(path).ok()?;
    let head = repo.head().ok()?;
    let shorthand = head.shorthand()?;

    // Check for dirty status
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts)).ok()?;

    let is_dirty = statuses.iter().any(|s| s.status() != git2::Status::CURRENT);

    let icon = if is_dirty { " " } else { " ✔" }; // nf-fa-warning, nf-fa-check

    Some(format!("{shorthand}{icon}"))
}

fn expand_cd_target(target: &str, cwd: &Path) -> PathBuf {
    if target == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = target.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    let path = Path::new(target);
    if path.is_absolute() {
        PathBuf::from(target)
    } else {
        cwd.join(target)
    }
}
