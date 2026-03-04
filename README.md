# snake-rs

A pastel-themed Snake game written in Rust with `macroquad`, now featuring:

- Pause + 3-second resume countdown
- Persistent per-mode/grid high scores
- Smooth speed ramp with configurable base speed
- Modes: Classic, Wrap, Zen (no-death overlap)
- In-game settings overlay
- Run summary stats on game over
- Timed combo scoring (`x1`..`x5`)
- Bundled audio (SFX + looping BGM)

## Build and run

```bash
cargo run
```

Release build:

```bash
cargo run --release
```

## Controls

- Move: Arrow keys or `W`, `A`, `S`, `D`
- Pause / unpause: `P`
- Open/close settings: `Tab`
- Apply pending mode/grid changes in settings: `Enter`
- Toggle theme: `M`
- Toggle FPS overlay: `F`
- Restart after game over: `R`
- Quit: `Q` or `Esc` (`Esc` closes settings first)

## Settings overlay

Settings fields:

- Game mode (Classic / Wrap / Zen)
- Grid preset (Small / Medium / Large)
- Base speed
- Difficulty ramp on/off
- Theme default
- Audio on/off
- Music volume
- SFX volume

Mode and grid changes are reset-required settings and are applied with `Enter`.

## Persistence

The game persists `settings` and per-mode-grid high scores in:

- macOS/Linux/Windows: OS-local app data path via `dirs::data_local_dir()`
- File name: `snake-rs/save.json`

If the file is missing or invalid, defaults are used and rewritten on the next save.

## Audio assets

Expected files under `assets/audio/`:

- `eat.wav`
- `death.wav`
- `pause.wav`
- `resume.wav`
- `bgm.ogg`

Missing or invalid audio files are non-fatal: gameplay continues without those sounds.

## Useful commands

```bash
cargo check
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```
