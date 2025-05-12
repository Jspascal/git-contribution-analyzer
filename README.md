# Git Contribution Analyzer

A TUI-based tool written in Rust for analyzing git repository contributions.

## Features

- Interactive TUI interface
- Detailed contribution statistics
- Navigate contributions with arrow keys
- Shows author details and commit metrics
- Export functionality for analysis results

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- Git (for analyzing repositories)

## Building from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/git-contribution-analyzer.git
   cd git-contribution-analyzer
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

   The binary will be available at `target/release/git-contribution-analyzer`

## Installation

### From Crates.io

```bash
cargo install git-contribution-analyzer
```

### From Local Build

```bash
cargo install --path .
```

## Basic Usage

### Analyze a Git Repository

```bash
git-contribution-analyzer --path /path/to/your/git/repository
```

### Available Command Line Options

```
USAGE:
    git-contribution-analyzer [OPTIONS] --path <PATH>

OPTIONS:
    -h, --help           Print help information
    -p, --path <PATH>    Path to the git repository to analyze
    -o, --output <PATH>  Optional: Export results to specified file (JSON format)
    -V, --version        Print version information
```

## Controls (TUI Interface)

- `↑`/`↓` : Navigate through contributor list
- `Enter` : View detailed stats for selected contributor
- `q` : Quit the application
- `?` : Show help dialog
