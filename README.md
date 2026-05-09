# gitinspector-rs

`gitinspector-rs` is a modern, fast, and robust statistical analysis tool for git repositories. It is a Rust-based port of the original Python `gitinspector`, designed for extreme performance and decoupling for future use in desktop applications.

## Features
- **Author Activity Heatmap** (v1.1): GitHub-style contribution grids with interactive tooltips.
- **CLI Diagnostics** (v1.1): Real-time progress bars and performance timing for large repository analysis.
- **Repo Health Audit** (v1.1): Automatic detection of branch sprawl (stale branches >90 days), PR throughput estimation, and large file (blob) auditing.
- **Hotspot Analysis** (v1.1): Identification of frequently changed files with actual LOC (Lines of Code) and size metrics.
- **Author Statistics**: Count insertions, deletions, and total commits per author.
- **Concurrent Blame Tracking**: Tracks active lines of code owned by each author with detailed file drill-downs.
- **Timeline Analysis**: Multi-axis visualization of commits, insertions, and deletions over time.
- **Filtering**: Advanced typed regex support for author, email, revision, message, and file paths.
- **Modern Reporting**: Export to HTML (with Chart.js), JSON (optimized for desktop apps), XML, Markdown, and Text.

## Project Structure
This is a Cargo Workspace:
- `core/`: The main library, containing all analysis logic and traits (`GitProvider`). Returns serializable structs.
- `apps/cli/`: The command-line interface executable.
- `docs/`: MyST-based documentation.

## Installation

### From Source (Recommended)
Make sure you have Rust and Cargo installed. You can install `gitinspector-rs` directly from the source:

```bash
# Clone the repository
git clone https://github.com/softmentor/gitinspector-rs.git
cd gitinspector-rs

# Install the binary to your cargo bin directory (~/.cargo/bin)
cargo install --path apps/cli
```

Once installed, you can run `gitinspector-rs` from anywhere in your terminal.

## Usage

```bash
# Basic usage against a local git repository (with progress feedback)
gitinspector-rs /path/to/repo --grading

# Generate a modern, interactive HTML report with Heatmap and Health Audit
gitinspector-rs /path/to/repo --grading -F html > report.html

# Generate a GitHub-ready Markdown report
gitinspector-rs /path/to/repo --grading -F markdown > report.md

# Export JSON for desktop/GUI tools
gitinspector-rs /path/to/repo --grading -F json > stats.json
```

## Documentation

To view the documentation, you can run MyST:
```bash
cd docs
myst start
```