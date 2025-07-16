// src/command.rs

use crate::error::AppResult;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum CommandUpdate {
    NewLine(String),
    Finished,
}

#[derive(Clone, Debug)]
pub struct CommandLog {
    pub command: String,
    pub output: String,
    pub is_running: bool,
    pub cwd: PathBuf,
}

impl CommandLog {
    pub fn new(command: String, output: String, is_running: bool, cwd: PathBuf) -> Self {
        Self {
            command,
            output,
            is_running,
            cwd,
        }
    }
}

#[derive(Default)]
pub struct CommandManager {
    kill_sender: Option<oneshot::Sender<()>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn_command(
        &mut self,
        cmd: &str,
        args: &[String],
        cwd: &Path,
        tx: UnboundedSender<CommandUpdate>,
    ) -> AppResult<()> {
        let mut child = TokioCommand::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let (kill_tx, mut kill_rx) = oneshot::channel();
        self.kill_sender = Some(kill_tx);

        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_out.send(CommandUpdate::NewLine(line)).is_err() {
                    break;
                }
            }
        });

        let tx_err = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_err
                    .send(CommandUpdate::NewLine(format!("[stderr] {}", line)))
                    .is_err()
                {
                    break;
                }
            }
        });

        let tx_finish = tx;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    status = child.wait() => {
                        // Command finished on its own
                        let _ = status;
                        break;
                    }
                    _ = &mut kill_rx => {
                        // Kill signal received
                        let _ = child.kill().await;
                        break;
                    }
                }
            }
            let _ = tx_finish.send(CommandUpdate::Finished);
        });

        Ok(())
    }

    pub fn kill_running_command(&mut self) -> AppResult<()> {
        if let Some(sender) = self.kill_sender.take() {
            // Send the kill signal. We don't care if it fails,
            // as that means the process already finished.
            let _ = sender.send(());
        }
        Ok(())
    }
}

use std::fs;

#[derive(Default)]
pub struct CompletionState {
    pub active: bool,
    pub suggestions: Vec<String>,
    pub selected_index: usize,
}

impl CompletionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_completion(&mut self, input_buffer: &str, cwd: &Path) {
        self.active = true;
        self.selected_index = 0;
        self.suggestions = self.generate_suggestions(input_buffer, cwd);
        if self.suggestions.is_empty() {
            self.active = false;
        }
    }

    pub fn stop_completion(&mut self) {
        self.active = false;
        self.suggestions.clear();
    }

    pub fn next_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    pub fn previous_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index =
                (self.selected_index + self.suggestions.len() - 1) % self.suggestions.len();
        }
    }

    pub fn apply_completion(&self, current_input: &str) -> Option<(String, usize)> {
        let suggestion = self.suggestions.get(self.selected_index)?;
        let mut parts: Vec<&str> = current_input.split_whitespace().collect();
        if !parts.is_empty() {
            parts.pop();
        }
        parts.push(suggestion);
        let mut new_input = parts.join(" ");

        let is_dir = suggestion.ends_with('/');
        if !is_dir {
            new_input.push(' ');
        }
        let new_cursor_pos = new_input.len();

        Some((new_input, new_cursor_pos))
    }

    fn generate_suggestions(&self, input_buffer: &str, cwd: &Path) -> Vec<String> {
        let partial = input_buffer.split_whitespace().last().unwrap_or("");
        let (base_dir, partial_name) = {
            let p = Path::new(partial);
            if partial.ends_with('/') || (p.is_dir() && partial.starts_with('/')) {
                (p, "")
            } else {
                (
                    p.parent().unwrap_or_else(|| Path::new("")),
                    p.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                )
            }
        };

        let search_path = cwd.join(base_dir);

        if let Ok(entries) = fs::read_dir(&search_path) {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let file_name_str = entry.file_name().to_string_lossy().to_string();
                    if file_name_str.starts_with(partial_name) {
                        let mut suggestion_path = base_dir.join(&file_name_str);
                        if entry.path().is_dir() {
                            suggestion_path.push("");
                        }
                        Some(suggestion_path.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
