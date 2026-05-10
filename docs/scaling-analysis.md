# Scaling and Memory Optimization Analysis

## 1. Problem Statement
When running `gitinspector-rs` on high-traffic repositories (e.g., `openclaw`), the application experiences massive memory spikes, often leading to OOM (Out Of Memory) kills or severe system degradation (thrashing).

## 2. Root Cause Analysis: The "Batch Processing" Trap
The current architecture follows a monolithic **Load -> Parse -> Aggregate** pattern. This creates three redundant memory buffers that scale linearly with the size of the Git history:

1.  **Raw Buffer**: `git log --numstat` output is read into a single `String` (`raw_output`). For 100k commits, this can exceed 500MB.
2.  **AST Buffer**: The `String` is parsed into a `Vec<Commit>`. Due to Rust's struct alignment and the nested `Vec<FileChange>`, this often doubles or triples the memory footprint of the raw text.
3.  **Aggregation State**: Statistics are computed only after the entire `Vec<Commit>` is loaded, meaning the peak memory usage is `Raw + AST + Stats`.

### Current Memory Complexity:
For large repositories, $N \gg M$, making the commit history the dominant (and unbounded) factor.

## 2b. The "Grading" and "Formatter" Traps (Post-Processing)
While commit streaming solves the history depth issue, the **Grading** and **Reporting** phases introduced new bottlenecks:

1.  **Process Explosion ($O(F)$)**: The `git blame` logic spawned a separate process for every tracked file ($F$) simultaneously. For 10k+ files, this exceeds system process limits and causes kernel-level hangs.
2.  **Quadratic Lookup ($O(M \times F)$)**: Updating file metrics used linear searches (`.find()`) inside loops over all files, leading to quadratic time complexity that pins the CPU.
3.  **Formatter Bloat**: The HTML reporter attempted to embed the complete activity history of all authors and files into a single DOM. Browsers cannot render 10k+ interactive rows without hanging.


---

## 3. Proposed Solution: The Streaming Pipeline
We will transition to a **Streaming Pipeline** architecture. This moves the complexity of history depth from $O(N)$ to $O(1)$ by processing commits as they arrive from the Git process.

### Architectural Shift:
- **Piped Stdout**: We use `Stdio::piped()` to read the Git output as a stream of lines.
- **Line-based Parser**: The parser will operate on a `BufReader`, emitting `Commit` objects via a channel (or iterator) as soon as they are fully formed.
- **Incremental Aggregator**: An aggregator will listen for `Commit` objects and update the `AuthorStats` and `FileStats` maps immediately.

### Optimized Memory Complexity:
- **History Depth**: $O(1)$ (only the current commit is in memory).
- **Repository Breadth**: $O(M)$ (Total authors and files must still be tracked).
- **Total**: $O(M)$

This ensures memory usage is almost constant regardless of whether the repo has 100 commits or 1,000,000 commits.

### Grading & Reporting Fixes:
- **Semaphore-limited Concurrency**: `git blame` processes are now governed by a concurrency semaphore (e.g., 16 concurrent workers), preventing system-wide resource exhaustion.
- **Hash-based Aggregation**: Replaced linear file lookups with `HashMap` $O(1)$ lookups during the metrics phase.
- **Report Pruning**: The HTML/Markdown formatters now implement "Top-N" pruning, showing only the most significant contributors and hotspots while providing aggregate summaries for the long tail.


---

## 4. Comparison of Options

| Option | Memory Usage | Implementation Effort | Pros | Cons |
| :--- | :--- | :--- | :--- | :--- |
| **Current (Batch)** | $O(N+M)$ | Low | Simple code, easy tests. | Crashes on large repos. |
| **Streaming (Proposed)** | $O(M)$ | Medium | Stable, works on any repo. | Requires async stream logic. |
| **SQLite Backend** | $O(1)$ | High | Minimal RAM, persistent. | Massive overhead, slow IO. |

**Why Streaming is best**: It provides the stability of a database-backed solution without the performance penalty of disk IO or the complexity of schema management.

---

## 5. Limitations
- **Sort-Dependent Metrics**: Some metrics (like "longest streak") require time-ordered data. Fortunately, `git log` is naturally ordered by time.
- **Aggregation Memory**: If a repository has millions of *unique files*, the $O(M)$ part could still become large. However, even 100k unique files typically fit well within modern RAM (approx. 100MB-200MB).

---

## 6. Synthetic Benchmarking Plan
To verify the fix without risking system hangs, we will:
1.  **Mock Git Provider**: Create a provider that generates millions of fake "Commit" lines at high speed.
2.  **Memory Monitor**: Use a test runner that tracks `Peak RSS` (Resident Set Size).
3.  **Validation**: Ensure that increasing the mock commit count from 10k to 1M results in < 5% increase in memory usage.
