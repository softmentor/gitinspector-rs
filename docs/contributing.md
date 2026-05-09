# Contributing

We welcome contributions to `gitinspector-rs`!

## Architecture

The project is a Rust Workspace containing:
- `core/`: The main library. This handles all I/O via the `GitProvider` trait and parses results into strictly typed `serde` structs.
- `apps/cli/`: The command-line interface logic.

## Building and Testing

To build the project:
```bash
cargo build
```

To run the unit tests:
```bash
cargo test
```

## Adding new features

If you add a new metric or flag, please ensure you update the `Config` struct in `core/src/config.rs` and the `Args` clap parser in `apps/cli/src/main.rs`.
