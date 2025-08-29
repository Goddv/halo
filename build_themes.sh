#!/bin/bash

# Create themes directory if it doesn't exist
mkdir -p themes

# Create individual theme files
cat > themes/cyber-nord.toml << 'EOF'
# Cyber-Nord Theme (Default)
primary = "#64B5FF"
accent = "#FF40A0"
warn = "#E7D98C"
error = "#FF5555"
success = "#64B5FF"
fg = "#DDE3EA"
bg = "#171A22"
comment = "#5A6473"
EOF

cat > themes/tokyo-night.toml << 'EOF'
# Tokyo Night Theme
primary = "#7AA2F7"
accent = "#BB9AF7"
warn = "#FFDA77"
error = "#F7768E"
success = "#7AA2F7"
fg = "#A9B1D6"
bg = "#1A1B26"
comment = "#6567A3"
EOF

cat > themes/catppuccin-mocha.toml << 'EOF'
# Catppuccin Mocha Theme
primary = "#89B4FA"
accent = "#F5C2E7"
warn = "#F9E2AF"
error = "#F38BA8"
success = "#89B4FA"
fg = "#CDD6F4"
bg = "#1E1E2E"
comment = "#6C7086"
EOF

cat > themes/nord.toml << 'EOF'
# Nord Theme
primary = "#88C0D0"
accent = "#ECEFF4"
warn = "#EBCB8B"
error = "#BF616A"
success = "#88C0D0"
fg = "#ECEFF4"
bg = "#2E3440"
comment = "#88C0D0"
EOF

cat > themes/solarized-dark.toml << 'EOF'
# Solarized Dark Theme
primary = "#268BD2"
accent = "#6C71C4"
warn = "#B58900"
error = "#DC322F"
success = "#268BD2"
fg = "#839496"
bg = "#002B36"
comment = "#586E75"
EOF

cat > themes/monokai.toml << 'EOF'
# Monokai Theme
primary = "#66D9EF"
accent = "#F92672"
warn = "#E6DB74"
error = "#F92672"
success = "#66D9EF"
fg = "#F8F8F2"
bg = "#272822"
comment = "#75715E"
EOF

cat > themes/dracula.toml << 'EOF'
# Dracula Theme
primary = "#6272A4"
accent = "#FF79C6"
warn = "#F1FA8C"
error = "#FF5555"
success = "#6272A4"
fg = "#F8F8F2"
bg = "#282A36"
comment = "#6272A4"
EOF

cat > themes/gruvbox-dark.toml << 'EOF'
# Gruvbox Dark Theme
primary = "#FABD2F"
accent = "#CC241D"
warn = "#FABD2F"
error = "#CC241D"
success = "#FABD2F"
fg = "#EBDBB2"
bg = "#1D2021"
comment = "#928374"
EOF

cat > themes/one-dark.toml << 'EOF'
# One Dark Theme
primary = "#61AFEF"
accent = "#C678DD"
warn = "#E5C07B"
error = "#E06C75"
success = "#61AFEF"
fg = "#ABB2BF"
bg = "#282C34"
comment = "#5C6370"
EOF

# Create the zip archive
zip -r themes.zip themes/

echo "Themes archive created: themes.zip"
echo "Available themes:"
ls themes/*.toml | sed 's/themes\///' | sed 's/\.toml$//'

