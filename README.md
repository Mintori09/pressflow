# pressflow

A Rust GUI application built with [Slint](https://slint.dev/).

## Requirements

- **Rust**: `1.91` or newer (Edition 2024)
- **System dependencies** (Linux / Wayland):
  - `libwayland-dev`
  - `libxkbcommon-dev`
  - `pkg-config`

On Debian/Ubuntu:

```bash
sudo apt-get install -y libwayland-dev libxkbcommon-dev pkg-config
```

## Build & Run

```bash
cargo build --release
cargo run --release
```

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## AI Disclosure

This project uses AI coding assistants (e.g., GitHub Copilot, Claude, Gemini) to aid development, documentation, and testing.
All AI-generated code is reviewed, tested, and maintained by human contributors.

If you submit a Pull Request assisted by AI:

- Ensure you understand every line of code you submit.
- Verify that all tests and linters pass.
- Briefly note any significant AI-assisted changes in your PR description.

## License

Licensed under [GPL-3.0](LICENSE).
