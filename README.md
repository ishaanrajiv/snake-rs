# snake-rs

A pastel-themed Snake game written in Rust with `macroquad`, now featuring:

- Pause + 3-second resume countdown
- Persistent settings + per-mode/grid high scores
- Smooth speed ramp with configurable base speed
- Smooth movement interpolation + buffered turn input
- Adaptive board layout for resizable windows
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

## Web build (WASM)

Build the web bundle locally:

```bash
bash scripts/build-web.sh
```

This creates `dist/` with:

- `index.html`
- `gl.js`
- `snake-rs.wasm`
- `assets/`

## GitHub + Vercel project flow

Use this for proper continuous deployments (not one-off CLI deploys):

1. Push this repo to GitHub (`main` branch).
2. In Vercel, import the GitHub repo as a new project.
3. Keep defaults from `vercel.json`:
   - Build command: `bash scripts/build-web.sh`
   - Output directory: `dist`
4. Vercel will auto-deploy:
   - Preview deployments for PRs
   - Production deployments for merges to `main`

## Controls

- Move: Arrow keys or `W`, `A`, `S`, `D`
- Pause / unpause: `P`
- Open/close settings: `Tab`
- Settings navigation: `Up`/`Down` (field), `Left`/`Right` (change value)
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

Supported BGM fallback:

- `bgm.wav` (used automatically if `bgm.ogg` is missing or fails to load)

Missing or invalid audio files are non-fatal: gameplay continues without those sounds.

## Credits

Audio source and license details are documented in [CREDITS.md](CREDITS.md).
The loader also checks paths near the executable, so release builds can still find assets when run outside the repo root.

## Useful commands

```bash
cargo check
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```
