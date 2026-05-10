# Usage Guide

`gitinspector-rs` is designed to be both simple for quick checks and powerful for deep repository audits.

## Installation

Ensure you have the latest binary in your PATH. You can build it from source using:
```bash
cargo build --release
```

## Basic Command

Run the tool against any local git repository path:
```bash
gitinspector-rs /path/to/repo
```

## Diagnostic Phases

The tool executes analysis in three distinct phases:
1.  **🔍 Analysis**: Parses commit history, computes author contributions, and generates activity trends.
2.  **🏥 Health Audit**: Scans for repository maintenance debt, including stale branches and oversized files.
3.  **🚚 Blame Analysis**: Performs multi-threaded `git blame` across the codebase to determine line-level ownership.

## Command Options

### Formatting & Output
- `-F, --format <FORMAT>`: Output type (`text`, `html`, `markdown`, `json`, `xml`). Default is `text`.
- `--grading`: A "macro" flag that enables all detailed analysis features (`-r`, `-T`, `-m`). Recommended for full audits.

### Filtering
- `-f, --file-types <EXTS>`: Comma-separated list of extensions to include (e.g., `rs,js,py`).
- `-x, --exclude <PATTERN>`: Exclude specific data. Patterns can be prefixed:
    - `author:John`: Exclude commits by John.
    - `file:tests/`: Exclude files in the tests directory.
    - `email:bot@`: Exclude specific email domains.
    - `message:chore`: Exclude commits with specific subject lines.

### Detailed Metrics
- `-r, --responsibilities`: Enable line-level ownership via `git blame`.
- `-T, --timeline`: Include weekly/monthly activity visualizations.
- `-m, --metrics`: Enable code complexity and size metrics for hotspot detection.

## Professional Examples

### 1. Generating a Stakeholder Dashboard
Create a comprehensive HTML report for a project review:
```bash
gitinspector-rs . --grading -F html > stats-report.html
```

### 2. Auditing Large Codebases (Text)
Focus only on core source files and exclude test data:
```bash
gitinspector-rs . -f rs,c,cpp -x "file:tests/" --responsibilities
```

### 3. CI/CD Integration (JSON)
Extract metrics for automated tracking:
```bash
gitinspector-rs . -F json > repo-metrics.json
```
