# Roadmap

This document outlines the future development plan for `gitinspector-rs`. Our goal is to maintain 100% feature parity with the original Python tool while expanding into modern platforms.

## v1.1 (UX & Visuals) - COMPLETED
- [x] **Author Activity Heatmap**: GitHub-style weekly contribution grid.
- [x] **CLI Diagnostics**: Real-time progress bars and analysis timing.
- [x] **Repo Health Audit**: Branch listing, PR estimation, and large blob detection.

## v1.2 (Advanced Metrics)
- **Stability/Age Metrics**: Port the legacy "Stability" (code survival rate) and "Age" (average code age) metrics.
- **Git Config Integration**: Support reading global and local `.gitconfig` settings for defaults.
- **Gravatar Support**: Optional integration to fetch and link author avatars.

## v2.0 (The Desktop Experience)
- **`apps/desktop`**: Initialize a Tauri-based desktop application.
- **Dynamic Charts**: Use the existing JSON output to render interactive charts in the desktop UI.
- **Real-time Analysis**: Background monitoring of repositories.

---

## Feature Comparison (Current Progress)

| Feature | Python Status | Rust Status |
| :--- | :--- | :--- |
| **Output Formats** | Yes | **Completed** |
| **Timeline** | Yes | **Completed** |
| **Author Heatmap** | No | **Completed (v1.1)** |
| **CLI Diagnostics** | No | **Completed (v1.1)** |
| **Repo Health Audit** | No | **Completed (v1.1)** |
| **Concurrent Blame** | Yes | **Completed** (Async) |
| **Filtering (Regex)** | Yes | **Completed** |
| **Grading Mode** | Yes | **Completed** |
| **Stability/Age** | Yes | Planned (v1.2) |
| **Gravatar** | Yes | Planned (v1.2) |
| **Git Config** | Yes | Planned (v1.2) |
| **Desktop App** | No | Planned (v2.0) |
