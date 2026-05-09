# Architecture

The application is structured into the following workspaces to ensure maximum decoupling and reusability:

- **`core`**: The main library crate that handles parsing Git repositories and calculating metrics. Everything is structured using `serde` to easily interact with web-based frontends.
- **`apps/cli`**: The CLI wrapper that allows you to run `gitinspector-rs` from the terminal.
- **`apps/desktop`** *(Future)*: Planned Tauri or Electron desktop UI.
- **`docs`**: This documentation site powered by MyST Markdown.
