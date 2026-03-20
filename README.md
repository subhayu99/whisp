# Whisp

Privacy-first keystroke suggestion daemon. Monitors your typing, captures context, and suggests better phrasing — all running locally on your machine.

## Features

- **Local-first**: Runs entirely on your device via Ollama. No data leaves your machine
- **Screenshot context**: Optionally captures your screen for richer suggestions (requires multimodal model)
- **Cross-platform**: macOS, Linux, Windows
- **Zero setup**: Auto-downloads and manages Ollama. Pick a model and go
- **Privacy by design**: No disk writes of keystrokes, no telemetry, no analytics. Sensitive buffers are cryptographically zeroed
- **Open source**: Fully auditable codebase with reproducible builds

## Quick Start

Download the latest release for your platform from [Releases](https://github.com/subhayu99/whisp/releases), run it, and pick a model.

### Build from source

```bash
# Prerequisites: Rust 1.75+, Node.js 18+
cargo build --release
```

## How It Works

1. Whisp monitors your keystrokes in the background
2. When you pause typing (~1.5s), it packages your text + optional screenshot
3. A local LLM suggests an improved version
4. The suggestion appears as ghost text near your cursor
5. Press **Tab** to accept, **Esc** to dismiss

## Models

| Model | Size | RAM | Multimodal | Best for |
|-------|------|-----|------------|----------|
| moondream:1.8b | ~1.1 GB | ~2 GB | Yes | Vision + text (recommended) |
| llava:7b | ~4.7 GB | ~8 GB | Yes | High-quality vision + text |
| qwen3:0.6b | ~523 MB | ~1 GB | No | Fast text-only suggestions |
| qwen3:1.7b | ~1.1 GB | ~2 GB | No | Better text-only quality |

> Text-only models skip the screenshot feature. Choose a multimodal model for full context-aware suggestions.

## Privacy

- Keystrokes and screenshots are **never written to disk** — processing is in-memory only
- All sensitive buffers are zeroed on drop using the `zeroize` crate
- Password fields are auto-detected and capture pauses immediately
- Password managers (1Password, Bitwarden, KeePassXC) are blocked by default
- Cloud LLM requires explicit opt-in with a clear data warning

## Architecture

```
Keystroke → Input Monitor → Privacy Guard → Context Engine → LLM Bridge → Overlay
```

Built with Rust + Tauri v2. See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## License

MIT
