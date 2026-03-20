# Contributing to Whisp

## Development Setup

1. Install Rust 1.75+ via [rustup](https://rustup.rs)
2. Install Node.js 18+ (for Tauri frontend)
3. Clone the repo and build:

```bash
git clone https://github.com/subhayu99/whisp.git
cd whisp
cargo build
```

### Platform dependencies

**macOS**: Xcode Command Line Tools (`xcode-select --install`)

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  libx11-dev libxdo-dev libxcb-randr0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxtst-dev libevdev-dev
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- No unsafe code without a safety comment

## Pull Requests

1. Fork and create a feature branch from `main`
2. Write tests for new functionality
3. Ensure `cargo test --workspace` passes
4. Ensure `cargo clippy -- -D warnings` is clean
5. Open a PR with a clear description of what and why

## Privacy Policy for Contributors

**Never** add code that:
- Writes keystroke data or screenshots to disk
- Sends data to external services without explicit user opt-in
- Adds telemetry, analytics, or crash reporting
- Logs raw keystroke content (log events, not content)
