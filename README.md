# grit-msg

`grit-msg` is a fast cross-platform Rust CLI that generates Conventional Commit messages from staged Git diffs using the Groq API.

## Features
- Reads staged changes with `git diff --cached`
- Summarizes large diffs to fit model context limits
- Interactive safety prompt: `y / n / copy`
- Copies generated message to clipboard (`copy`)
- Secure key storage with keyring, plus resilient local fallback

## Requirements
- `git`
- A Groq API key: https://console.groq.com/keys

## Install (No Rust Required)
1. Go to GitHub Releases for this repo.
2. Download the archive matching your OS/CPU.
3. Extract and place `grit-msg` (`grit-msg.exe` on Windows) in your `PATH`.

Release archives are published automatically when a tag like `v0.1.0` is pushed.

## Build
```bash
cargo build --release --locked
```

Binary:
```bash
target/release/grit-msg
```

## Usage
```bash
grit-msg
```

Options:
```bash
grit-msg --model llama-3.3-70b-versatile --diff-token-budget 6000
grit-msg --clear-aiapi-keys
```

Notes:
- If no staged changes exist, the tool exits with an error.
- `--clear-aiapi-keys` removes stored keys from keyring + fallback files.
- `GROQ_API_KEY` environment variable is supported and takes precedence.

## Development
```bash
cargo fmt --all
cargo check
cargo test
```

## Maintainer Release
```bash
git tag v0.1.0
git push origin v0.1.0
```
