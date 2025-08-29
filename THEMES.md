# Halo Theme System

Halo uses a zip-based theme system that automatically extracts themes on first run and allows for easy theme management.

## How It Works

1. **First Run**: When Halo starts for the first time, it automatically extracts a built-in themes archive (`themes.zip`) to `~/.config/halo/themes/`
2. **Theme Discovery**: The shell automatically discovers all `.toml` files in the themes directory
3. **Dynamic Loading**: Themes are loaded from files at runtime, no code changes needed

## Built-in Themes

The following themes are included by default:

- `cyber-nord` - Cyberpunk-inspired Nord theme (default)
- `tokyo-night` - Tokyo Night color scheme
- `catppuccin-mocha` - Catppuccin Mocha variant
- `nord` - Classic Nord theme
- `solarized-dark` - Solarized Dark theme
- `monokai` - Monokai color scheme
- `dracula` - Dracula theme
- `gruvbox-dark` - Gruvbox Dark theme
- `one-dark` - One Dark theme

## Commands

### List Available Themes
```bash
theme list
```

### Set a Theme
```bash
theme set <theme-name>
```

Or for interactive selection:
```bash
theme set
```
Then use arrow keys (↑/↓) to navigate and Enter to select.

### Refresh Themes
```bash
theme refresh
```
This command re-extracts the themes archive, useful after installing new themes.

## Interactive Theme Selection

When you run `theme set` without specifying a theme name, Halo enters interactive theme selection mode:

- **Navigation**: Use ↑/↓ arrow keys to browse themes
- **Live Preview**: Themes are applied immediately as you navigate
- **Visual Feedback**: Current selection is highlighted with accent color
- **Confirmation**: Press Enter to select, Esc to cancel
- **Auto-save**: Selected theme is automatically saved to your session

## Adding Custom Themes

1. Create a `.toml` file in `~/.config/halo/themes/`
2. Use the following format:

```toml
# Theme Name
primary = "#RRGGBB"    # Primary accent color
accent = "#RRGGBB"     # Secondary accent color
warn = "#RRGGBB"       # Warning color
error = "#RRGGBB"      # Error color
success = "#RRGGBB"    # Success color
fg = "#RRGGBB"         # Foreground text color
bg = "#RRGGBB"         # Background color
comment = "#RRGGBB"    # Comment/muted text color
```

3. Restart Halo or use `theme refresh` to load the new theme

## Building the Themes Archive

To update the built-in themes:

1. Modify theme files in the `themes/` directory
2. Run `./build_themes.sh` to regenerate `themes.zip`
3. Rebuild Halo

## File Locations

- **Themes Directory**: `~/.config/halo/themes/`
- **Theme Archive**: Embedded in the binary (`themes.zip`)
- **User Config**: `~/.config/halo/halo.toml`
- **Session Data**: `~/.config/halo/session.json`
