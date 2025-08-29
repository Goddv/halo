// src/state.rs

use crate::command::CommandLog;
use crate::completion::CompletionState;
use crate::error::AppResult;
use crate::themes;
use ratatui::style::Color;
#[derive(Clone)]
pub struct UiConfig {
    pub scrollbar_thumb: String,
    pub prompt: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            scrollbar_thumb: "█".to_string(),
            prompt: "❯".to_string(),
        }
    }
}
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;
#[derive(Clone)]
pub struct Theme {
    pub primary: Color,
    pub accent: Color,
    pub warn: Color,
    pub error: Color,
    pub success: Color,
    pub fg: Color,
    pub bg: Color,
    pub comment: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Softer, more readable defaults
            primary: Color::Rgb(100, 181, 255),   // #64B5FF
            accent: Color::Rgb(255, 64, 160),     // #FF40A0
            warn: Color::Rgb(231, 217, 140),      // #E7D98C
            error: Color::Rgb(255, 85, 85),       // #FF5555
            success: Color::Rgb(100, 181, 255),   // #64B5FF
            fg: Color::Rgb(221, 227, 234),        // #DDE3EA
            bg: Color::Rgb(23, 26, 34),           // #171A22
            comment: Color::Rgb(90, 100, 115),    // #5A6473
        }
    }
}

impl Theme {
    fn parse_color(input: &str) -> Option<Color> {
        let s = input.trim();
        // Hex: #RRGGBB or #RGB
        if let Some(hex) = s.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::Rgb(r, g, b));
            } else if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let r = (r << 4) | r;
                let g = (g << 4) | g;
                let b = (b << 4) | b;
                return Some(Color::Rgb(r, g, b));
            }
        }
        // rgb(r,g,b)
        if let Some(body) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
            let parts: Vec<_> = body.split(',').map(|x| x.trim()).collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].parse().ok()?;
                let g: u8 = parts[1].parse().ok()?;
                let b: u8 = parts[2].parse().ok()?;
                return Some(Color::Rgb(r, g, b));
            }
        }
        // 8-bit indexed: ansi:N or index:N
        if let Some(num) = s.strip_prefix("ansi:").or_else(|| s.strip_prefix("index:")) {
            if let Ok(v) = num.parse::<u8>() {
                return Some(Color::Indexed(v));
            }
        }
        // Named colors
        let name = s.to_ascii_lowercase();
        let named = match name.as_str() {
            "black" => Color::Black,
            "white" => Color::White,
            "gray" | "grey" => Color::Gray,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" | "purple" => Color::Magenta,
            "cyan" => Color::Cyan,
            _ => return None,
        };
        Some(named)
    }

    pub fn from_table(tbl: &toml::value::Table, base: Theme) -> Theme {
        let mut t = base;
        if let Some(v) = tbl.get("primary").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.primary = c;
            }
        }
        if let Some(v) = tbl.get("accent").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.accent = c;
            }
        }
        if let Some(v) = tbl.get("warn").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.warn = c;
            }
        }
        if let Some(v) = tbl.get("error").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.error = c;
            }
        }
        if let Some(v) = tbl.get("success").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.success = c;
            }
        }
        if let Some(v) = tbl.get("fg").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.fg = c;
            }
        }
        if let Some(v) = tbl.get("bg").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.bg = c;
            }
        }
        if let Some(v) = tbl.get("comment").and_then(|v| v.as_str()) {
            if let Some(c) = Self::parse_color(v) {
                t.comment = c;
            }
        }
        t
    }

    pub fn from_name(name: &str) -> Theme {
        match name {
            // A vibrant cyberpunk + nord fusion (current default)
            "cyber-nord" => Theme::default(),
            "dracula" => Theme {
                primary: Color::Rgb(98, 114, 164),
                accent: Color::Rgb(255, 121, 198),
                warn: Color::Rgb(241, 250, 140),
                error: Color::Rgb(255, 85, 85),
                success: Color::Rgb(98, 114, 164),
                fg: Color::Rgb(248, 248, 242),
                bg: Color::Rgb(40, 42, 54),
                comment: Color::Rgb(98, 114, 164),
            },
            "gruvbox-dark" => Theme {
                primary: Color::Rgb(250, 189, 47),
                accent: Color::Rgb(204, 36, 29),
                warn: Color::Rgb(250, 189, 47),
                error: Color::Rgb(204, 36, 29),
                success: Color::Rgb(250, 189, 47),
                fg: Color::Rgb(235, 219, 178),
                bg: Color::Rgb(29, 32, 33),
                comment: Color::Rgb(146, 131, 116),
            },
            "one-dark" => Theme {
                primary: Color::Rgb(97, 175, 239),
                accent: Color::Rgb(198, 120, 221),
                warn: Color::Rgb(229, 192, 123),
                error: Color::Rgb(224, 108, 117),
                success: Color::Rgb(97, 175, 239),
                fg: Color::Rgb(171, 178, 191),
                bg: Color::Rgb(40, 44, 52),
                comment: Color::Rgb(92, 99, 112),
            },
            _ => Theme::default(),
        }
    }
}

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
    pub aliases: std::collections::HashMap<String, String>,
    // Reserved for future: drive highlight from state rather than recomputing
    // pub active_preview_index: Option<usize>,
    _last_start_time: Option<Instant>,
    pub theme: Theme,
    pub theme_name: String,
    pub ui: UiConfig,
    // Theme selection mode
    pub theme_selection_mode: bool,
    pub available_themes: Vec<String>,
    pub theme_selection_index: usize,
}

impl State {
    pub fn new() -> AppResult<Self> {
        let cwd = std::env::current_dir()?;
        let mut state = Self {
            should_quit: false,
            needs_redraw: true,
            username: users::get_current_username()
                .and_then(|name| name.into_string().ok())
                .unwrap_or_else(|| "user".to_string()),
            cwd: cwd.clone(),
            git_branch: None,
            input_buffer: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
            command_log: vec![CommandLog::new(
                "".into(),
                "Welcome to Halo! A modern shell for a modern age.".into(),
                false,
                cwd,
            )],
            scroll_offset: 0,
            completion_state: CompletionState::new(),
            aliases: Default::default(),
            _last_start_time: None,
            theme: Theme::default(),
            theme_name: "cyber-nord".to_string(),
            ui: UiConfig::default(),
            // Theme selection mode
            theme_selection_mode: false,
            available_themes: Vec::new(),
            theme_selection_index: 0,
        };
        state.load_history()?;
        state.load_config();
        let _ = state.load_session();
        Ok(state)
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
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
            // Keep the newest HISTORY_LIMIT entries by draining from the front
            let excess = self.command_log.len() - HISTORY_LIMIT;
            self.command_log.drain(0..excess);
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

    // runtime fields for duration tracking
    #[allow(dead_code)]
    fn now() -> Instant {
        Instant::now()
    }
    pub fn mark_last_log_started(&mut self) {
        // Store start time in a sidecar map keyed by index if needed; easiest is to stash in output
        // but we will track via a local Instant until finish and compute delta. Use a hidden field on State.
        self._last_start_time = Some(Self::now());
    }

    pub fn finish_last_log_with_result(&mut self, exit_code: Option<i32>) {
        if let Some(last) = self.command_log.last_mut() {
            last.is_running = false;
            last.exit_code = exit_code;
            if let Some(start) = self._last_start_time.take() {
                let elapsed = start.elapsed().as_millis();
                last.duration_ms = Some(elapsed);
            }
            self.needs_redraw = true;
        }
    }

    fn history_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("halo/history");
            p
        })
    }

    pub fn load_history(&mut self) -> AppResult<()> {
        if let Some(path) = Self::history_path() {
            if let Ok(file) = fs::File::open(&path) {
                let reader = BufReader::new(file);
                self.history = serde_json::from_reader(reader).unwrap_or_default();
            }
        }
        Ok(())
    }

    pub fn save_history(&self) -> AppResult<()> {
        if let Some(path) = Self::history_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let file = fs::File::create(&path)?;
            serde_json::to_writer_pretty(file, &self.history)?;
        }
        Ok(())
    }

    pub fn load_config(&mut self) {
        // Read minimal halo.toml from config dir, parse aliases table if present
        if let Some(mut path) = dirs::config_dir() {
            // Ensure base dir exists
            path.push("halo");
            let _ = fs::create_dir_all(&path);
            // Config file path
            path.push("halo.toml");
            if let Ok(text) = fs::read_to_string(&path) {
                if let Ok(value) = text.parse::<toml::Value>() {
                    if let Some(aliases) = value.get("aliases").and_then(|v| v.as_table()) {
                        self.aliases = aliases
                            .iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect();
                    }
                    if let Some(theme_name) = value.get("theme").and_then(|v| v.as_str()) {
                        if !self.load_theme_from_file(theme_name) {
                            // Fallback to built-in theme if file not found
                            self.theme = Theme::from_name(theme_name);
                        }
                        self.theme_name = theme_name.to_string();
                    } else if let Some(theme_tbl) = value.get("theme").and_then(|v| v.as_table()) {
                        self.theme = Theme::from_table(theme_tbl, self.theme.clone());
                        self.theme_name = "custom".to_string();
                    }

                    if let Some(ui_tbl) = value.get("ui").and_then(|v| v.as_table()) {
                        if let Some(sym) = ui_tbl.get("scrollbar_thumb").and_then(|v| v.as_str()) {
                            self.ui.scrollbar_thumb = sym.to_string();
                        }
                        if let Some(sym) = ui_tbl.get("prompt").and_then(|v| v.as_str()) {
                            self.ui.prompt = sym.to_string();
                        }
                    }
                }
            } else {
                // Create a starter config with current (softened) defaults
                let default_cfg = format!(
                    "# Halo config – created on first run\n# Set a named theme or define [theme] colors.\n# Available names: cyber-nord, dracula, gruvbox-dark, one-dark\n\n# theme = \"cyber-nord\"\n\n[theme]\nprimary = \"#64B5FF\"\naccent  = \"#FF40A0\"\nwarn    = \"#E7D98C\"\nerror   = \"#FF5555\"\nfg      = \"#DDE3EA\"\nbg      = \"#171A22\"\ncomment = \"#5A6473\"\n\n[ui]\nscrollbar_thumb = \"█\"\nprompt = \"❯\"\n\n# [aliases]\n# ll = \"ls -alF\"\n# gs = \"git status\"\n"
                );
                let _ = fs::write(&path, default_cfg);
            }
        }
        
        // Extract themes from archive if needed
        if let Err(e) = themes::extract_themes_if_needed() {
            eprintln!("Warning: Failed to extract themes: {}", e);
        }
    }

    fn session_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("halo/session.json");
            p
        })
    }

    pub fn load_session(&mut self) -> AppResult<()> {
        if let Some(path) = Self::session_path() {
            if let Ok(file) = fs::File::open(&path) {
                let reader = BufReader::new(file);
                #[derive(Deserialize)]
                struct Session {
                    last_cwd: String,
                    last_theme_name: Option<String>,
                }
                if let Ok(session) = serde_json::from_reader::<_, Session>(reader) {
                    let candidate = PathBuf::from(session.last_cwd);
                    if candidate.is_dir() {
                        if let Err(_e) = std::env::set_current_dir(&candidate) {
                            // ignore failure, keep current cwd
                        }
                        self.cwd = candidate;
                    }
                    if let Some(name) = session.last_theme_name {
                        self.theme = Theme::from_name(&name);
                        self.theme_name = name;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_session(&self) -> AppResult<()> {
        if let Some(path) = Self::session_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            #[derive(Serialize)]
            struct Session {
                last_cwd: String,
                last_theme_name: String,
            }
            let data = Session {
                last_cwd: self.cwd.to_string_lossy().to_string(),
                last_theme_name: self.theme_name.clone(),
            };
            let file = fs::File::create(&path)?;
            serde_json::to_writer_pretty(file, &data)?;
        }
        Ok(())
    }



    pub fn get_available_themes(&self) -> Vec<String> {
        let mut themes = Vec::new();
        
        if let Some(mut themes_dir) = dirs::config_dir() {
            themes_dir.push("halo/themes");
            if let Ok(entries) = fs::read_dir(themes_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Some(extension) = entry.path().extension() {
                        if extension == "toml" {
                            if let Some(stem) = entry.path().file_stem() {
                                if let Some(name) = stem.to_str() {
                                    themes.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        themes.sort();
        themes
    }

    pub fn load_theme_from_file(&mut self, theme_name: &str) -> bool {
        if let Some(mut theme_path) = dirs::config_dir() {
            theme_path.push(format!("halo/themes/{}.toml", theme_name));
            
            if let Ok(content) = fs::read_to_string(theme_path) {
                if let Ok(value) = content.parse::<toml::Value>() {
                    if let Some(theme_tbl) = value.as_table() {
                        self.theme = Theme::from_table(theme_tbl, Theme::default());
                        self.theme_name = theme_name.to_string();
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn enter_theme_selection_mode(&mut self) {
        self.theme_selection_mode = true;
        self.available_themes = self.get_available_themes();
        self.theme_selection_index = 0;
        self.needs_redraw = true;
    }

    pub fn exit_theme_selection_mode(&mut self) {
        self.theme_selection_mode = false;
        self.available_themes.clear();
        self.theme_selection_index = 0;
        self.needs_redraw = true;
    }

    pub fn select_theme_up(&mut self) {
        if self.theme_selection_mode && !self.available_themes.is_empty() {
            self.theme_selection_index = self.theme_selection_index.saturating_sub(1);
            self.preview_selected_theme();
            self.needs_redraw = true;
        }
    }

    pub fn select_theme_down(&mut self) {
        if self.theme_selection_mode && !self.available_themes.is_empty() {
            self.theme_selection_index = (self.theme_selection_index + 1) % self.available_themes.len();
            self.preview_selected_theme();
            self.needs_redraw = true;
        }
    }

    pub fn confirm_theme_selection(&mut self) -> bool {
        if self.theme_selection_mode && !self.available_themes.is_empty() {
            let theme_name = self.available_themes[self.theme_selection_index].clone();
            if self.load_theme_from_file(&theme_name) {
                self.theme_name = theme_name;
                let _ = self.save_session();
                self.exit_theme_selection_mode();
                return true;
            }
        }
        false
    }

    pub fn preview_selected_theme(&mut self) {
        if self.theme_selection_mode && !self.available_themes.is_empty() {
            if let Some(theme_name) = self.available_themes.get(self.theme_selection_index) {
                // Temporarily load the theme for preview without changing the theme_name
                if let Some(mut theme_path) = dirs::config_dir() {
                    theme_path.push(format!("halo/themes/{}.toml", theme_name));
                    
                    if let Ok(content) = fs::read_to_string(theme_path) {
                        if let Ok(value) = content.parse::<toml::Value>() {
                            if let Some(theme_tbl) = value.as_table() {
                                self.theme = Theme::from_table(theme_tbl, Theme::default());
                            }
                        }
                    }
                }
            }
        }
    }


}
