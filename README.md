<p align="center">
  <img src="https://raw.githubusercontent.com/Goddv/halo/main/.github/assets/icon.png" width="150" alt="Halo Shell Logo">
  <h1 align="center">⚡ Halo Shell ⚡</h1>
</p>

<p align="center">
  A next-gen terminal shell for the modern developer. Built in Rust with a focus on speed, style, and productivity.
</p>

<p align="center">
  <!-- GitHub Actions Badge -->
  <a href="https://github.com/Goddv/halo/actions/workflows/rust.yml">
    <img src="https://github.com/Goddv/halo/actions/workflows/rust.yml/badge.svg" alt="Build Status">
  </a>
  <!-- License Badge -->
  <a href="https://github.com/Goddv/halo/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
  </a>
  <!-- Version Badge -->
  <a href="#">
    <img src="https://img.shields.io/badge/version-0.1.0-brightgreen" alt="Version">
  </a>
  <!-- Platform Badge -->
  <a href="#">
    <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey" alt="Platform">
  </a>
</p>

---

**Halo** is a command-line shell reimagined from the ground up. Frustrated with the limitations of traditional terminals, Halo leverages the power of **Rust**, **Tokio**, and **Ratatui** to provide an incredibly fast, non-blocking, and visually rich experience. Its bottom-up, block-based interface makes tracking command history more intuitive than ever.

## ✨ Features

*   🚀 **Blazingly Fast & Asynchronous**: Built in Rust on top of Tokio, ensuring a non-blocking UI that remains responsive even while commands are running.
*   🎨 **Modern Bottom-Up TUI**: A chat-like interface where commands flow upwards. No more endless scrolling to find where a command's output begins.
*   🧠 **Context-Aware Autocompletion**: `Tab` completion for system executables, built-ins, and file paths. It's smart enough to suggest *only* directories after `cd`.
*   📜 **Intuitive History Preview**: Seamlessly scroll up through your history. Each command and its complete output is treated as a single, easily reviewable block.
*   🔧 **Git Integration**: The status bar instantly shows your current Git branch and status (✔ for clean,  for dirty).
*   💅 **Stylish & Cohesive Design**: A beautiful "Cyber-Nord" theme that is both modern and easy on the eyes. Every component has been styled for a uniform and professional look.

## 📸 Screenshots

<p align="center">
  <strong>Active Prompt with Git Status and Running Command</strong><br>
  <img src="https://github.com/Goddv/halo/blob/main/.github/assets/input_file_0.png?raw=true" alt="Halo Shell Active Prompt">
</p>

<p align="center">
  <strong>History Preview Mode</strong><br>
  <em>Scrolling up highlights previous commands and their output, turning the input area into a preview pane.</em><br>
  <img src="https://github.com/Goddv/halo/blob/main/.github/assets/input_file_1.png?raw=true" alt="Halo Shell History Preview">
</p>


## 🛠️ Installation

Halo is built with standard Rust tooling.

#### Prerequisites
*   [Rust and Cargo](https://www.rust-lang.org/tools/install)
*   A Nerd Font (for icons like ``, `📁`, `⚡`) is highly recommended for the best visual experience. [FiraCode Nerd Font](https://www.nerdfonts.com/font-downloads) is a great choice.

#### Building from Source
1.  Clone the repository:
    ```sh
    git clone https://github.com/Goddv/halo.git
    ```
2.  Navigate to the project directory:
    ```sh
    cd halo
    ```
3.  Build and run for development:
    ```sh
    cargo run
    ```
4.  For the best performance, build a release binary:
    ```sh
    cargo build --release
    ```
    The executable will be located at `target/release/halo-shell`.

---

## ⌨️ Usage & Keybindings

| Key(s)           | Action                                                                                             |
|------------------|----------------------------------------------------------------------------------------------------|
| **Typing Text**  | Enters commands. If you are in history preview mode, this will instantly exit it and start a new command. |
| **Tab**            | Activates context-aware autocompletion. Cycles through suggestions if the menu is open.            |
| **Enter**          | Executes the current command.                                                                      |
| **Ctrl+C**         | Kills the currently running command or exits completion menu.                                        |
| **Mouse Wheel**    | Scrolls up and down through the command history, activating preview mode.                           |
| **PageUp/PageDown**| Scrolls through history in larger steps.                                                           |
| **Up/Down Arrow**  | Navigates through command history (only when not in preview mode).                                   |
| **Esc**            | Exits the completion menu.                                                                         |

## 🚀 Roadmap

Halo is an emerging project with an ambitious vision. Here's what's planned for the future:

- [ ] **Advanced Configuration** (`halo.toml` for themes, keybindings, etc.)
- [ ] **Plugin System** (allow users to extend Halo with Rust)
- [ ] **Shell Scripting Engine**
- [ ] **Multi-line and block editing**
- [ ] **Tabbed Interface** for multiple concurrent sessions
- [ ] **Alias Support**

## ❤️ Contributing

Contributions are what make the open-source community such an amazing place. Any contributions you make are **greatly appreciated**.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

Please also feel free to open an [issue](https://github.com/Goddv/halo/issues) with any bugs or feature requests!

## 📄 License

This project is licensed under the MIT License. See the `LICENSE` file for details.
