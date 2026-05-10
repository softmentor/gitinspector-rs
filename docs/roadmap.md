# Project Roadmap

The mission of `gitinspector-rs` is to provide the most performant and auditable repository analytics engine in the ecosystem. 

## v1.1.0: Foundation & Diagnostics (LATEST)
Status: **STABLE RELEASE**

The focus of v1.1.0 was on auditability and professional diagnostics.
- [x] **Repository Health Audit**: Identification of stale branches and large file blobs.
- [x] **High-Fidelity HTML Reports**: Modern, responsive dashboard with Chart.js integration.
- [x] **Performance Metadata**: Tracking of analysis duration and engine versioning.
- [x] **Concurrent Execution**: Asynchronous `git blame` and metric calculation.

## v1.2.0: Advanced Analytics
Status: **IN DEVELOPMENT**

The upcoming release aims to deepen the analytical capabilities of the engine.
- **Code Survival (Stability)**: Metrics for tracking how long code stays in the repository before being modified.
- **Code Age Metrics**: Average age of lines per author and file.
- **Git Ecosystem Integration**: Support for `.gitconfig` alias resolution and Gravatar profile integration.
- **Export Hardening**: PDF export support for executive reporting.

## v2.0.0: The Desktop Experience
Status: **PLANNED**

Transformation from a CLI tool to a cross-platform analytics workstation.
- **Tauri Native GUI**: A fluid desktop interface for multi-repository monitoring.
- **Interactive Drill-downs**: Clickable charts to explore commit details and churn history.
- **Custom SQL Queries**: Support for querying repository data using a SQL-like interface.

---

## Feature Parity & Innovations

| Feature | Legacy (Python) | Rust (v1.1.0+) |
| :--- | :--- | :--- |
| **Output Formats** | Basic | **Advanced (HTML/MD/JSON)** |
| **Analysis Performance** | Standard | **Ultra-Fast (Async/Rust)** |
| **Author Heatmap** | No | **Yes** |
| **Repo Health Audit** | No | **Yes** |
| **Performance Metadata** | No | **Yes** |
| **Concurrent Blame** | No | **Yes** |
| **Filtering (Regex)** | Yes | **Yes** |
| **Stability/Age** | Yes | Planned (v1.2) |
| **Desktop App** | No | Planned (v2.0) |
