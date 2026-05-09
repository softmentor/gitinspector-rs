# Usage

`gitinspector-rs` is a high-performance CLI tool for gathering deep statistics about a git repository.

## Basic Execution

Run the CLI targeting a repository. By default, it provides a summary of commit activity.
```bash
gitinspector-rs /path/to/repo
```

## CLI Feedback (v1.1)

When running the tool, you will see real-time progress indicators:
- **[1/3] Analyzing**: Parsing commit history and computing base metrics.
- **[2/3] Auditing**: Checking repository health, identifying stale branches (>90 days), and auditing large blobs.
- **[3/3] Blame Analysis**: Executing concurrent `git blame` with a progress bar and ETA.

## Options

- `-F`, `--format`: Output format. Can be `text`, `json`, `xml`, `html`, or `markdown` (default: `text`).
- `-f`, `--file-types`: A comma separated list of file extensions to include (e.g. `rs,js,py`).
- `-x`, `--exclude`: An exclusion pattern. Supports prefixes: `author:`, `email:`, `revision:`, `message:`, `file:`.
- `--grading`: Enable all detailed reports (Heatmap, Timeline, Blame, Metrics).
- `-T`, `--timeline`: Shows a grouped timeline of activity.
- `-r`, `--responsibilities`: Runs `git blame` concurrently to track active line ownership.
- `-m`, `--metrics`: Enables code complexity metrics.

## Example

Generate a full HTML dashboard for the current directory:
```bash
gitinspector-rs . --grading -F html > report.html
```
