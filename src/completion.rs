// src/completion.rs

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt; // For checking executable permission on Unix-like systems
use std::path::{Component, Path, PathBuf};

// An enum to determine what kind of paths we should suggest.
#[derive(PartialEq)]
enum PathFilter {
    All,
    DirectoriesOnly,
}

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

    /// Triggers the new, context-aware completion logic.
    pub fn start_completion(&mut self, input_buffer: &str, cwd: &Path) {
        self.active = true;
        self.selected_index = 0;
        self.suggestions = self.generate_suggestions(input_buffer, cwd);

        // If there's only one suggestion, apply it immediately.
        if self.suggestions.len() == 1 {
            // We need a mutable buffer to apply, but we can't get one here.
            // This is a candidate for a future enhancement. For now, show the menu.
        }

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

    /// Applies the selected suggestion to the input buffer.
    pub fn apply_completion(&self, current_input: &str) -> Option<(String, usize)> {
        let suggestion = self.suggestions.get(self.selected_index)?;

        // Find the start of the word being completed.
        let mut last_word_start = current_input
            .rfind(char::is_whitespace)
            .map_or(0, |i| i + 1);
        if current_input.is_empty() {
            last_word_start = 0;
        }

        // Reconstruct the input string with the completion.
        let mut new_input = current_input[..last_word_start].to_string();
        new_input.push_str(suggestion);

        // Add a space after the completion unless it's a directory.
        if !suggestion.ends_with('/') {
            new_input.push(' ');
        }

        let new_cursor_pos = new_input.len();

        Some((new_input, new_cursor_pos))
    }

    /// The new context-aware suggestion generation engine.
    fn generate_suggestions(&self, input_buffer: &str, cwd: &Path) -> Vec<String> {
        let words: Vec<&str> = input_buffer.split_whitespace().collect();

        // The token to complete is the last "word", unless the line ends with a space.
        let token_to_complete = if input_buffer.ends_with(' ') {
            ""
        } else {
            words.last().unwrap_or(&"")
        };

        // Determine if we are typing the very first word (the command).
        let is_completing_command =
            words.is_empty() || (words.len() == 1 && !input_buffer.ends_with(' '));

        if is_completing_command {
            self.suggest_executables(token_to_complete)
        } else {
            // It's an argument, so complete a path.
            let command = words.first().unwrap_or(&"");
            let filter = match *command {
                "cd" => PathFilter::DirectoriesOnly,
                _ => PathFilter::All, // Most commands take files or directories
            };
            self.suggest_paths(token_to_complete, cwd, filter)
        }
    }

    /// Suggests executables from the system's $PATH.
    fn suggest_executables(&self, partial_cmd: &str) -> Vec<String> {
        let mut commands = std::collections::HashSet::new();
        // Add built-ins
        for cmd in ["cd", "pwd", "exit"] {
            if cmd.starts_with(partial_cmd) {
                commands.insert(cmd.to_string());
            }
        }

        if let Ok(path_var) = env::var("PATH") {
            for path in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.filter_map(Result::ok) {
                        if let Ok(metadata) = entry.metadata() {
                            // On Unix, check the executable permission bit.
                            let is_executable = metadata.permissions().mode() & 0o111 != 0;
                            if metadata.is_file() && is_executable {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.starts_with(partial_cmd) {
                                        commands.insert(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_commands: Vec<String> = commands.into_iter().collect();
        sorted_commands.sort();
        sorted_commands
    }

    /// Suggests file or directory paths.
    fn suggest_paths(&self, partial_path: &str, cwd: &Path, filter: PathFilter) -> Vec<String> {
        // Handle home directory expansion
        let mut path_to_complete = PathBuf::new();
        if partial_path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                path_to_complete.push(home);
                // Add the rest of the path, skipping the tilde
                path_to_complete.push(&partial_path[1..]);
            }
        } else {
            path_to_complete.push(partial_path);
        }

        let (search_dir, partial_name) = if partial_path.ends_with('/') || partial_path == "~" {
            (cwd.join(&path_to_complete), "")
        } else {
            (
                cwd.join(&path_to_complete)
                    .parent()
                    .unwrap_or(cwd)
                    .to_path_buf(),
                path_to_complete
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(""),
            )
        };

        if let Ok(entries) = fs::read_dir(&search_dir) {
            let mut results: Vec<String> = entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(partial_name) {
                        // Check if the entry matches the filter (All or Dirs only)
                        let file_type = entry.file_type().ok()?;
                        let is_dir = file_type.is_dir();
                        if filter == PathFilter::DirectoriesOnly && !is_dir {
                            return None;
                        }

                        // Determine the base path of the token being completed
                        let mut suggestion_base = PathBuf::from(partial_path);
                        if suggestion_base.file_name().is_some() {
                            suggestion_base.pop();
                        }

                        let mut final_suggestion = suggestion_base.join(file_name);

                        if is_dir {
                            final_suggestion.push(""); // Appends a trailing slash
                        }

                        Some(final_suggestion.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect();
            results.sort();
            return results;
        }

        Vec::new()
    }
}
