# snake-rs

A pastel-themed Snake game written in Rust with `macroquad`.

## Prerequisites

- A terminal
- Internet access (for first-time Rust toolchain install)

## Install Rust and Cargo

Cargo is installed together with Rust via `rustup`.

### macOS / Linux

Run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then reload your shell (or run the command shown by the installer), and verify:

```bash
rustc --version
cargo --version
```

### Windows

1. Download and run `rustup-init.exe` from:
   - https://rustup.rs
2. Follow the default installation steps.
3. Open a new PowerShell window and verify:

```powershell
rustc --version
cargo --version
```

## Clone and enter the project

```bash
git clone <your-repo-url>
cd snake-rs
```

## Build

Debug build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

## Run

Run in debug mode:

```bash
cargo run
```

Run optimized release build:

```bash
cargo run --release
```

## Controls

- Move: Arrow keys or `W`, `A`, `S`, `D`
- Restart after game over: `R`
- Quit: `Q` or `Esc`

## Useful Cargo commands

Check compilation without building a binary:

```bash
cargo check
```

Format code:

```bash
cargo fmt
```

Lint with Clippy:

```bash
cargo clippy --all-targets --all-features
```

Run tests:

```bash
cargo test
```

## Build artifacts

- Debug binaries are placed under `target/debug/`
- Release binaries are placed under `target/release/`

