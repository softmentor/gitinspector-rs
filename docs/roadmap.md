# Roadmap

This document outlines the future development plan for `gitinspector-rs`. Our goal is to maintain 100% feature parity with the original Python tool while expanding into modern platforms.

## v1.x (CLI Parity & Polish)

- **Advanced Blame Metrics**: Port the "Stability" (code survival rate) and "Age" (average code age) metrics.
- **Git Config Integration**: Support reading global and local `.gitconfig` settings for defaults (e.g., `inspector.format`).
- **Gravatar Support**: Optional integration to fetch and link author avatars in HTML and XML reports.
- **Native Git Support**: Optional support for the `git2` (libgit2) crate as an alternative to the `CliGitProvider`.

## v2.0 (The Desktop Experience)

- **`apps/desktop`**: Initialize a Tauri-based desktop application.
- **Dynamic Charts**: Use the existing JSON output to render interactive charts (D3.js or Chart.js) in the desktop UI.
- **Real-time Analysis**: Background monitoring of repositories to provide live statistics.

## Long-term Vision

- **Internationalization (i18n)**: Localization support for multiple languages.
- **Deep Metrics**: Advanced static analysis beyond simple cyclomatic complexity (e.g., maintainability index, dependency analysis).
- **Mobile App**: A companion app for monitoring repository health on the go.

---

## Feature Comparison (Current Progress)

| Feature | Python Status | Rust Status |
| :--- | :--- | :--- |
| **Output Formats** | Yes | **Completed** |
| **Timeline** | Yes | **Completed** |
| **Concurrent Blame** | Yes (Threaded) | **Completed** (Async) |
| **Filtering (Regex)** | Yes | **Completed** |
| **Grading Mode** | Yes | **Completed** |
| **Stability/Age** | Yes | Planned (v1.1) |
| **Gravatar** | Yes | Planned (v1.1) |
| **Git Config** | Yes | Planned (v1.2) |
| **Desktop App** | No | Planned (v2.0) |
