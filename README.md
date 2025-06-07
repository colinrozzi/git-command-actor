# git-command-actor

A Theater actor that executes arbitrary git commands and returns structured results.

## Overview

The git-command-actor is a simple, focused WebAssembly component designed to run any git command in a specified repository and return structured results. It's designed as a building block for more complex git automation workflows.

## Features

- Execute any git command with arbitrary arguments
- Repository path validation
- Configurable timeout handling
- Structured output with stdout, stderr, exit codes, and timing
- Proper error handling and validation
- Clean shutdown with comprehensive results

## Usage

### Configuration

Create an `init.json` file with your git command configuration:

```json
{
  "repository_path": "/path/to/your/git/repo",
  "git_args": ["status", "--porcelain"],
  "timeout_seconds": 30,
  "working_directory": "/optional/different/wd"
}
```

### Configuration Options

- **repository_path** (required): Path to the git repository
- **git_args** (required): Array of git command arguments (e.g., `["status", "--porcelain"]`)
- **timeout_seconds** (optional): Command timeout in seconds (default: 30)
- **working_directory** (optional): Working directory for the command (defaults to repository_path)

### Build and Run

1. **Build the actor:**
   ```bash
   cargo component build --release --target wasm32-unknown-unknown
   ```

2. **Start the actor:**
   ```bash
   theater start manifest.toml
   ```

### Output Structure

The actor shuts down with a JSON result containing:

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "M  file1.txt\nA  file2.txt\n",
  "stderr": "",
  "command": ["git", "-C", "/repo", "status", "--porcelain"],
  "execution_time_ms": 150,
  "error": null,
  "repository_path": "/path/to/repo"
}
```

## Example Use Cases

### Check Repository Status
```json
{
  "repository_path": "/my/repo",
  "git_args": ["status", "--porcelain"]
}
```

### Get Recent Commits
```json
{
  "repository_path": "/my/repo",
  "git_args": ["log", "--oneline", "-10"]
}
```

### Stage All Changes
```json
{
  "repository_path": "/my/repo",
  "git_args": ["add", "."]
}
```

### Create a Commit
```json
{
  "repository_path": "/my/repo",
  "git_args": ["commit", "-m", "Add new feature"]
}
```

### Get Diff
```json
{
  "repository_path": "/my/repo",
  "git_args": ["diff", "--cached"]
}
```

### Push Changes
```json
{
  "repository_path": "/my/repo",
  "git_args": ["push", "origin", "main"]
}
```

## Error Handling

The actor handles several types of errors:

1. **Validation Errors**: Repository doesn't exist, isn't a directory, etc.
2. **Process Errors**: Failed to spawn git process
3. **Git Errors**: Git command returns non-zero exit code
4. **Timeout Errors**: Command exceeds configured timeout

All errors are captured in the result's `error` field and `success` is set to `false`.

## Building Blocks for Complex Workflows

This actor is designed to be a simple building block. More complex git workflows can be built by chaining multiple git-command-actor instances together. For example:

1. Check status → Stage changes → Commit → Push
2. Create branch → Switch to branch → Make changes → Commit
3. Fetch → Merge → Push

## Requirements

- Git must be installed and available in the PATH
- Repository must be a valid git repository
- Appropriate file system permissions for the repository

## Theater Handlers

This actor uses the following Theater handlers:

- **runtime**: For logging and shutdown
- **process**: For executing git commands
- **timing**: For timeout handling

## Comparison to commit-actor

While the commit-actor is specialized for creating AI-generated commits, git-command-actor is a general-purpose tool that can execute any git command. The commit-actor could be refactored to use multiple git-command-actor instances for its workflow steps.
