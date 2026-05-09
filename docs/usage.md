# Usage

`gitinspector-rs` can be executed from the terminal to gather statistics about a git repository.

## Basic Execution

Run the CLI targeting a repository:
```bash
gitinspector-cli /path/to/repo
```

## Options

- `-F`, `--format`: Output format. Can be `text`, `json`, `xml`, `html`, or `markdown` (default: `text`).
- `-f`, `--file-types`: A comma separated list of file extensions to include (e.g. `rs,js,py`).
- `-x`, `--exclude`: An exclusion pattern. Supports prefixes: `author:`, `email:`, `revision:`, `message:`, `file:`.
- `--grading`: Enable grading mode (equivalent to `-HlmrTw` in the original tool).
- `-T`, `--timeline`: Shows a grouped timeline of commits, insertions, and deletions.
- `-r`, `--responsibilities`: Runs `git blame` concurrently to track active line ownership and file-level drill-downs.
- `-m`, `--metrics`: Enables basic code metrics like cyclomatic complexity.

## Example

```bash
gitinspector-cli . -T -r -x "^tests/"
```
