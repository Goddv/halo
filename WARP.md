# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Halo is a next-generation terminal shell built in Rust with a focus on speed, style, and productivity. It features a bottom-up, block-based TUI interface where commands flow upwards like a chat interface, built using Tokio for async operations and Ratatui for the terminal UI.

## Development Commands

### Building and Running
```bash
# Development build and run
cargo run

# Release build (recommended for testing performance)
cargo build --release
./target/release/halo-shell

# Check compilation without running
cargo check

# Clean build artifacts
cargo clean
```

### Code Quality and Formatting
```bash
# Format code
cargo fmt

# Run linter (Clippy)
cargo clippy

# Run with all warnings
cargo clippy -- -W clippy::all
```

### Testing
```bash
# Run tests (currently no tests in the codebase)
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Dependency Management
```bash
# Update dependencies
cargo update

# Add a new dependency
cargo add <crate_name>

# Show dependency tree
cargo tree
```

## Architecture Overview

### Core Components

**Main Architecture Pattern**: Event-driven async architecture using Tokio
- `main.rs`: Entry point with terminal setup and cleanup guards
- `app.rs`: Main application loop and command orchestration
- `state.rs`: Central state management including UI config, themes, and user data
- `event.rs`: Event handling system for user input
- `ui.rs`: Ratatui-based UI rendering

**Command Execution System**:
- `command.rs`: Async command execution using Tokio processes
- Commands run in separate tasks with stdout/stderr streaming
- Real-time output updates via mpsc channels
- Process management with kill capability

**Completion System**:
- `completion.rs`: Context-aware tab completion
- Executable completion from $PATH for commands
- Directory-only completion for `cd` command
- Path completion for file arguments

**Theme System**:
- `themes.rs`: Theme loading and management
- TOML-based theme configuration in `themes/` directory
- Runtime theme switching with preview mode
- Color parsing for hex, RGB, ANSI, and named colors

### Key Data Structures

- `State`: Central application state including current directory, git status, command history, and UI configuration
- `CommandLog`: Individual command execution records with timing and output
- `CompletionState`: Tab completion state and suggestion management
- `Theme`: Color scheme definitions with parsing from TOML files

### Configuration System

- Configuration directory: `~/.config/halo/`
- Main config: `halo.toml` (aliases, theme selection, UI customization)
- Session persistence: `session.json` (last directory, theme)
- Command history: `history` (JSON format)
- Themes: `themes/*.toml` (color definitions)

### Built-in Commands

The shell includes several built-in commands handled directly in `app.rs`:
- `exit`: Quit the shell
- `cd`: Change directory with home expansion
- `pwd`: Print working directory  
- `theme`: Theme management (list, set, refresh)
- `alias`: Alias listing (expansion from config)
- `:reload`: Reload configuration

## Development Guidelines

### Code Organization
- Each module has a clear single responsibility
- Async operations use Tokio consistently
- Error handling uses `anyhow` for application errors
- UI state is centralized in the `State` struct

### Terminal UI (Ratatui)
- All UI rendering happens in `ui.rs`
- State changes trigger `needs_redraw` flag
- Terminal setup includes proper cleanup guards
- Bottom-up interface with command blocks

### Theme Development
- Themes are TOML files in `themes/` directory
- Support hex colors (`#RRGGBB`), RGB (`rgb(r,g,b)`), ANSI indexed, and named colors
- Theme preview mode available during selection
- Default theme is "cyber-nord"

### Command Execution
- All external commands run via Tokio async processes
- Real-time output streaming to UI
- Proper process cleanup on shell exit
- Support for process interruption (Ctrl+C)

### Git Integration
- Git status displayed in status bar
- Branch name with clean/dirty indicators
- Automatic git repository detection

## File Structure

```
src/
├── main.rs          # Entry point and terminal setup
├── app.rs           # Main application loop and command handling
├── state.rs         # State management and configuration
├── event.rs         # Input event handling
├── ui.rs            # Terminal UI rendering (Ratatui)
├── command.rs       # Async command execution
├── completion.rs    # Tab completion system
├── themes.rs        # Theme management
└── error.rs         # Error handling types

themes/              # Theme definitions
├── cyber-nord.toml  # Default theme
├── dracula.toml     # Dracula color scheme
├── gruvbox-dark.toml
└── [other themes]
```

## Configuration Files

- `~/.config/halo/halo.toml`: Main configuration with aliases and theme settings
- `~/.config/halo/session.json`: Session persistence (working directory, theme)
- `~/.config/halo/history`: Command history in JSON format
- `~/.config/halo/themes/`: User theme definitions

## Key Dependencies

- `tokio`: Async runtime for command execution
- `ratatui`: Terminal UI framework  
- `crossterm`: Cross-platform terminal control
- `git2`: Git repository integration
- `serde`/`serde_json`: Configuration serialization
- `toml`: Theme and config file parsing
- `shlex`: Shell-style argument parsing
- `dirs`: System directory discovery

## Performance Considerations

- The shell is designed for speed with async command execution
- UI updates are throttled to avoid excessive redraws
- Command history is limited to 100 entries to prevent memory growth
- Built in Rust for memory safety and performance

## Testing Strategy

Currently, the project has no automated tests. When adding tests, consider:
- Unit tests for completion logic
- Integration tests for command execution
- UI component testing with mock states
- Theme parsing and color conversion tests
