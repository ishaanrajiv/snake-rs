mod audio;
mod game;
mod persistence;
mod settings;

use macroquad::prelude::*;

use crate::audio::AudioManager;
use crate::game::{Direction, Game, HighScoreKey, Point};
use crate::persistence::{HighScoreMap, load_persisted, save_persisted};
use crate::settings::{
    ALL_SETTING_FIELDS, AdjustmentDirection, SettingField, Settings, ThemeMode, adjust_setting,
    format_field_value,
};

const PERF_SAMPLE_SECONDS: f32 = 1.0;
const RESUME_COUNTDOWN_SECONDS: f32 = 3.0;
#[cfg(not(target_arch = "wasm32"))]
const TARGET_RENDER_FPS: f64 = 300.0;
#[cfg(not(target_arch = "wasm32"))]
const TARGET_FRAME_SECONDS: f64 = 1.0 / TARGET_RENDER_FPS;

#[derive(Clone, Copy)]
enum SessionState {
    Running,
    Paused,
    ResumeCountdown { remaining: f32 },
    GameOver,
}

#[derive(Clone, Copy)]
enum MenuReturnState {
    Running,
    Paused,
    GameOver,
}

struct SettingsMenu {
    open: bool,
    selected_index: usize,
    draft: Settings,
    return_state: MenuReturnState,
}

impl SettingsMenu {
    fn new(settings: &Settings) -> Self {
        Self {
            open: false,
            selected_index: 0,
            draft: settings.clone(),
            return_state: MenuReturnState::Paused,
        }
    }

    fn open(&mut self, settings: &Settings, return_state: MenuReturnState) {
        self.open = true;
        self.selected_index = 0;
        self.draft = settings.clone();
        self.return_state = return_state;
    }

    fn close(&mut self) -> MenuReturnState {
        self.open = false;
        self.return_state
    }

    fn selected_field(&self) -> SettingField {
        ALL_SETTING_FIELDS[self.selected_index]
    }

    fn move_selection(&mut self, delta: i32) {
        let len = ALL_SETTING_FIELDS.len() as i32;
        let index = (self.selected_index as i32 + delta).rem_euclid(len);
        self.selected_index = index as usize;
    }

    fn has_pending_reset_fields(&self, active: &Settings) -> bool {
        self.draft.mode != active.mode || self.draft.grid_preset != active.grid_preset
    }

    fn field_pending(&self, active: &Settings, field: SettingField) -> bool {
        match field {
            SettingField::Mode => self.draft.mode != active.mode,
            SettingField::GridPreset => self.draft.grid_preset != active.grid_preset,
            _ => false,
        }
    }
}

struct BoardLayout {
    x: f32,
    y: f32,
    cell: f32,
    width_px: f32,
    height_px: f32,
}

struct PerformanceOverlay {
    show_fps: bool,
    sample_start_time: f64,
    sample_frames: u32,
    fps_display: f32,
}

impl PerformanceOverlay {
    fn new(now: f64) -> Self {
        Self {
            show_fps: false,
            sample_start_time: now,
            sample_frames: 0,
            fps_display: 0.0,
        }
    }

    fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
    }

    fn tick(&mut self, now: f64) {
        self.sample_frames += 1;
        let sample_elapsed = (now - self.sample_start_time) as f32;

        if sample_elapsed >= PERF_SAMPLE_SECONDS {
            let frames = self.sample_frames.max(1) as f32;
            self.fps_display = frames / sample_elapsed;
            self.sample_start_time = now;
            self.sample_frames = 0;
        }
    }
}

#[derive(Clone, Copy)]
struct Theme {
    background: Color,
    blob_a: Color,
    blob_b: Color,
    blob_c: Color,
    board_shell: Color,
    board_fill: Color,
    board_border: Color,
    grid_line: Color,
    title: Color,
    hud_text: Color,
    controls_text: Color,
    food_glow: Color,
    food: Color,
    body_start: Color,
    body_end: Color,
    head: Color,
    eye: Color,
    overlay: Color,
    overlay_title: Color,
    overlay_subtitle: Color,
    settings_panel: Color,
    settings_active: Color,
    settings_pending: Color,
}

fn pastel(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn pastel_alpha(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

fn lerp_color(start: Color, end: Color, t: f32) -> Color {
    let clamped = t.clamp(0.0, 1.0);
    Color::new(
        start.r + (end.r - start.r) * clamped,
        start.g + (end.g - start.g) * clamped,
        start.b + (end.b - start.b) * clamped,
        start.a + (end.a - start.a) * clamped,
    )
}

fn theme(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => Theme {
            background: pastel(248, 246, 255),
            blob_a: pastel_alpha(255, 212, 226, 0.34),
            blob_b: pastel_alpha(206, 234, 255, 0.32),
            blob_c: pastel_alpha(216, 244, 221, 0.28),
            board_shell: pastel_alpha(255, 255, 255, 0.72),
            board_fill: pastel(255, 252, 250),
            board_border: pastel(206, 199, 235),
            grid_line: pastel_alpha(213, 214, 226, 0.45),
            title: pastel(126, 112, 170),
            hud_text: pastel(111, 129, 166),
            controls_text: pastel(134, 141, 160),
            food_glow: pastel_alpha(255, 172, 162, 0.4),
            food: pastel(250, 144, 136),
            body_start: pastel(168, 226, 198),
            body_end: pastel(188, 170, 232),
            head: pastel(113, 196, 173),
            eye: pastel(66, 89, 100),
            overlay: pastel_alpha(255, 255, 255, 0.62),
            overlay_title: pastel(124, 108, 170),
            overlay_subtitle: pastel(111, 129, 166),
            settings_panel: pastel_alpha(255, 255, 255, 0.9),
            settings_active: pastel_alpha(222, 227, 255, 0.9),
            settings_pending: pastel(214, 129, 120),
        },
        ThemeMode::Dark => Theme {
            background: pastel(24, 27, 38),
            blob_a: pastel_alpha(149, 105, 176, 0.28),
            blob_b: pastel_alpha(110, 157, 206, 0.24),
            blob_c: pastel_alpha(108, 176, 142, 0.24),
            board_shell: pastel_alpha(9, 12, 19, 0.62),
            board_fill: pastel(33, 38, 53),
            board_border: pastel(110, 123, 175),
            grid_line: pastel_alpha(133, 148, 193, 0.3),
            title: pastel(209, 195, 242),
            hud_text: pastel(184, 197, 235),
            controls_text: pastel(153, 165, 201),
            food_glow: pastel_alpha(255, 129, 126, 0.36),
            food: pastel(255, 102, 116),
            body_start: pastel(122, 194, 171),
            body_end: pastel(148, 133, 208),
            head: pastel(102, 177, 160),
            eye: pastel(231, 236, 255),
            overlay: pastel_alpha(11, 15, 25, 0.72),
            overlay_title: pastel(216, 204, 246),
            overlay_subtitle: pastel(188, 201, 239),
            settings_panel: pastel_alpha(22, 27, 41, 0.94),
            settings_active: pastel_alpha(83, 97, 145, 0.55),
            settings_pending: pastel(255, 138, 118),
        },
    }
}

fn board_layout(grid_width: i32, grid_height: i32) -> BoardLayout {
    let sw = screen_width();
    let sh = screen_height();
    let side_margin = 84.0;
    let top_space = 150.0;
    let bottom_space = 88.0;

    let cell_x = (sw - side_margin * 2.0) / grid_width as f32;
    let cell_y = (sh - top_space - bottom_space) / grid_height as f32;
    let cell = cell_x.min(cell_y).max(8.0);
    let width_px = cell * grid_width as f32;
    let height_px = cell * grid_height as f32;
    let x = (sw - width_px) * 0.5;
    let y = top_space + ((sh - top_space - bottom_space) - height_px) * 0.5;

    BoardLayout {
        x,
        y,
        cell,
        width_px,
        height_px,
    }
}

fn handle_direction_input(game: &mut Game) {
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        game.set_direction(Direction::Up);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        game.set_direction(Direction::Down);
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        game.set_direction(Direction::Left);
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        game.set_direction(Direction::Right);
    }
}

fn draw_background(theme: &Theme) {
    clear_background(theme.background);
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;

    draw_circle(sw * 0.1 + t.sin() * 14.0, sh * 0.2, sh * 0.28, theme.blob_a);
    draw_circle(
        sw * 0.86 + t.cos() * 18.0,
        sh * 0.24,
        sh * 0.24,
        theme.blob_b,
    );
    draw_circle(
        sw * 0.7 + (t * 0.7).sin() * 11.0,
        sh * 0.84,
        sh * 0.34,
        theme.blob_c,
    );
}

fn draw_board(layout: &BoardLayout, theme: &Theme, grid_width: i32, grid_height: i32) {
    draw_rectangle(
        layout.x - 16.0,
        layout.y - 16.0,
        layout.width_px + 32.0,
        layout.height_px + 32.0,
        theme.board_shell,
    );
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.board_fill,
    );
    draw_rectangle_lines(
        layout.x - 1.5,
        layout.y - 1.5,
        layout.width_px + 3.0,
        layout.height_px + 3.0,
        3.0,
        theme.board_border,
    );

    for x in 1..grid_width {
        let px = layout.x + x as f32 * layout.cell;
        draw_line(
            px,
            layout.y,
            px,
            layout.y + layout.height_px,
            1.0,
            theme.grid_line,
        );
    }
    for y in 1..grid_height {
        let py = layout.y + y as f32 * layout.cell;
        draw_line(
            layout.x,
            py,
            layout.x + layout.width_px,
            py,
            1.0,
            theme.grid_line,
        );
    }
}

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn draw_hud(
    game: &Game,
    layout: &BoardLayout,
    performance: &PerformanceOverlay,
    theme_mode: ThemeMode,
    theme: &Theme,
    high_score: u32,
) {
    let title = "Snake";
    let title_dim = measure_text(title, None, 54, 1.0);
    draw_text_ex(
        title,
        (screen_width() - title_dim.width) * 0.5,
        layout.y - 62.0,
        TextParams {
            font_size: 54,
            color: theme.title,
            ..Default::default()
        },
    );

    let combo = game.combo_state().current_multiplier;
    let combo_text = if combo > 1 {
        format!("    Combo: x{combo}")
    } else {
        String::new()
    };

    let info = if performance.show_fps {
        format!(
            "Score: {}    High: {}    Mode: {}{}    FPS: {:.0}    Theme: {}",
            game.score(),
            high_score,
            game.mode().label(),
            combo_text,
            performance.fps_display,
            theme_mode.label()
        )
    } else {
        format!(
            "Score: {}    High: {}    Mode: {}{}    Theme: {}",
            game.score(),
            high_score,
            game.mode().label(),
            combo_text,
            theme_mode.label(),
        )
    };
    draw_text_ex(
        &info,
        layout.x,
        layout.y - 18.0,
        TextParams {
            font_size: 28,
            color: theme.hud_text,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Move: arrows/WASD | Pause: P | Settings: Tab | Restart: R (game over) | Theme: M | FPS: F | Quit: Q / Esc",
        layout.x,
        layout.y + layout.height_px + 42.0,
        TextParams {
            font_size: 21,
            color: theme.controls_text,
            ..Default::default()
        },
    );
}

fn draw_game(game: &Game, layout: &BoardLayout, step_progress: f32, theme: &Theme) {
    let food = game.food();
    let food_center_x = layout.x + food.x as f32 * layout.cell + layout.cell * 0.5;
    let food_center_y = layout.y + food.y as f32 * layout.cell + layout.cell * 0.5;
    let pulse = (get_time() as f32 * 4.0).sin() * 0.06 + 1.0;
    let food_radius = layout.cell * 0.27 * pulse;
    draw_circle(
        food_center_x,
        food_center_y,
        food_radius + 2.0,
        theme.food_glow,
    );
    draw_circle(food_center_x, food_center_y, food_radius, theme.food);

    let segments: Vec<Point> = game.snake().iter().copied().collect();
    let max_idx = (segments.len().saturating_sub(1)).max(1) as f32;
    let safe_progress = if game.next_move_would_collide() {
        0.0
    } else {
        step_progress
    };

    for idx in (1..segments.len()).rev() {
        let segment = segments[idx];
        let target = segments[idx - 1];
        let interp_x = lerp_f32(segment.x as f32, target.x as f32, safe_progress);
        let interp_y = lerp_f32(segment.y as f32, target.y as f32, safe_progress);
        let t = idx as f32 / max_idx;
        let color = lerp_color(theme.body_start, theme.body_end, t);
        let inset = layout.cell * 0.12;
        let x = layout.x + interp_x * layout.cell + inset;
        let y = layout.y + interp_y * layout.cell + inset;
        let size = layout.cell - inset * 2.0;
        draw_rectangle(x, y, size, size, color);
    }

    if let Some(head) = segments.first().copied() {
        let preview_dir = game.preview_direction();
        let (dx, dy) = preview_dir.delta();
        let head_target_x = head.x as f32 + dx as f32;
        let head_target_y = head.y as f32 + dy as f32;
        let interp_head_x = lerp_f32(head.x as f32, head_target_x, safe_progress);
        let interp_head_y = lerp_f32(head.y as f32, head_target_y, safe_progress);
        let inset = layout.cell * 0.09;
        let x = layout.x + interp_head_x * layout.cell + inset;
        let y = layout.y + interp_head_y * layout.cell + inset;
        let size = layout.cell - inset * 2.0;
        draw_rectangle(x, y, size, size, theme.head);

        let eye_offset = layout.cell * 0.18;
        let eye_radius = layout.cell * 0.05;
        let (ex1, ey1, ex2, ey2) = match preview_dir {
            Direction::Up => (
                x + eye_offset,
                y + eye_offset,
                x + size - eye_offset,
                y + eye_offset,
            ),
            Direction::Down => (
                x + eye_offset,
                y + size - eye_offset,
                x + size - eye_offset,
                y + size - eye_offset,
            ),
            Direction::Left => (
                x + eye_offset,
                y + eye_offset,
                x + eye_offset,
                y + size - eye_offset,
            ),
            Direction::Right => (
                x + size - eye_offset,
                y + eye_offset,
                x + size - eye_offset,
                y + size - eye_offset,
            ),
        };
        draw_circle(ex1, ey1, eye_radius, theme.eye);
        draw_circle(ex2, ey2, eye_radius, theme.eye);
    }
}

fn draw_pause_overlay(layout: &BoardLayout, theme: &Theme) {
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.overlay,
    );
    let headline = "Paused";
    let headline_dim = measure_text(headline, None, 62, 1.0);
    draw_text_ex(
        headline,
        layout.x + (layout.width_px - headline_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 - 10.0,
        TextParams {
            font_size: 62,
            color: theme.overlay_title,
            ..Default::default()
        },
    );

    let sub = "Press P to resume";
    let sub_dim = measure_text(sub, None, 28, 1.0);
    draw_text_ex(
        sub,
        layout.x + (layout.width_px - sub_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 + 30.0,
        TextParams {
            font_size: 28,
            color: theme.overlay_subtitle,
            ..Default::default()
        },
    );
}

fn draw_resume_overlay(layout: &BoardLayout, theme: &Theme, remaining: f32) {
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.overlay,
    );

    let count = remaining.ceil().clamp(1.0, 3.0) as i32;
    let text = format!("{count}");
    let dim = measure_text(&text, None, 110, 1.0);
    draw_text_ex(
        &text,
        layout.x + (layout.width_px - dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 + dim.height * 0.5,
        TextParams {
            font_size: 110,
            color: theme.overlay_title,
            ..Default::default()
        },
    );

    let sub = "Get ready";
    let sub_dim = measure_text(sub, None, 28, 1.0);
    draw_text_ex(
        sub,
        layout.x + (layout.width_px - sub_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 + 60.0,
        TextParams {
            font_size: 28,
            color: theme.overlay_subtitle,
            ..Default::default()
        },
    );
}

fn format_duration(seconds: f32) -> String {
    let seconds = seconds.max(0.0);
    let minutes = (seconds / 60.0).floor() as u32;
    let rem = (seconds % 60.0).floor() as u32;
    format!("{minutes:02}:{rem:02}")
}

fn draw_game_over_overlay(game: &Game, layout: &BoardLayout, theme: &Theme, high_score: u32) {
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.overlay,
    );

    let headline = if game.is_won() {
        "You Win"
    } else {
        "Game Over"
    };
    let headline_dim = measure_text(headline, None, 62, 1.0);
    draw_text_ex(
        headline,
        layout.x + (layout.width_px - headline_dim.width) * 0.5,
        layout.y + 92.0,
        TextParams {
            font_size: 62,
            color: theme.overlay_title,
            ..Default::default()
        },
    );

    let stats = game.run_stats();
    let lines = [
        format!("Final Score: {}", game.score()),
        format!("High Score: {high_score}"),
        format!(
            "Time Survived: {}",
            format_duration(stats.elapsed_seconds())
        ),
        format!("Foods Eaten: {}", stats.foods_eaten),
        format!("Max Combo: x{}", stats.max_combo.max(1)),
        format!("Avg Foods/Min: {:.1}", stats.avg_foods_per_minute),
    ];

    let mut y = layout.y + 150.0;
    for line in lines {
        draw_text_ex(
            &line,
            layout.x + 36.0,
            y,
            TextParams {
                font_size: 30,
                color: theme.overlay_subtitle,
                ..Default::default()
            },
        );
        y += 38.0;
    }

    let sub = "Press R to restart";
    let sub_dim = measure_text(sub, None, 30, 1.0);
    draw_text_ex(
        sub,
        layout.x + (layout.width_px - sub_dim.width) * 0.5,
        layout.y + layout.height_px - 28.0,
        TextParams {
            font_size: 30,
            color: theme.overlay_subtitle,
            ..Default::default()
        },
    );
}

fn draw_settings_overlay(
    layout: &BoardLayout,
    theme: &Theme,
    menu: &SettingsMenu,
    active: &Settings,
) {
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.overlay,
    );

    let panel_w = (layout.width_px * 0.82)
        .max(560.0)
        .min(screen_width() - 120.0);
    let panel_h = (layout.height_px * 0.82)
        .max(420.0)
        .min(screen_height() - 120.0);
    let panel_x = (screen_width() - panel_w) * 0.5;
    let panel_y = (screen_height() - panel_h) * 0.5;

    draw_rectangle(panel_x, panel_y, panel_w, panel_h, theme.settings_panel);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, theme.board_border);

    let title = "Settings";
    draw_text_ex(
        title,
        panel_x + 24.0,
        panel_y + 46.0,
        TextParams {
            font_size: 44,
            color: theme.overlay_title,
            ..Default::default()
        },
    );

    let mut y = panel_y + 94.0;
    for (index, field) in ALL_SETTING_FIELDS.iter().enumerate() {
        let is_selected = index == menu.selected_index;
        let row_h = 40.0;
        if is_selected {
            draw_rectangle(
                panel_x + 18.0,
                y - 28.0,
                panel_w - 36.0,
                row_h,
                theme.settings_active,
            );
        }

        let pending = menu.field_pending(active, *field);
        let mut value = format_field_value(&menu.draft, *field);
        if pending {
            value.push_str(" (press Enter)");
        }

        draw_text_ex(
            field.label(),
            panel_x + 32.0,
            y,
            TextParams {
                font_size: 28,
                color: theme.overlay_subtitle,
                ..Default::default()
            },
        );
        draw_text_ex(
            &value,
            panel_x + panel_w * 0.44,
            y,
            TextParams {
                font_size: 28,
                color: if pending {
                    theme.settings_pending
                } else {
                    theme.overlay_subtitle
                },
                ..Default::default()
            },
        );
        y += row_h;
    }

    let footer =
        "Navigate: Up/Down  |  Change: Left/Right  |  Apply Mode/Grid: Enter  |  Close: Tab / Esc";
    draw_text_ex(
        footer,
        panel_x + 22.0,
        panel_y + panel_h - 18.0,
        TextParams {
            font_size: 22,
            color: theme.controls_text,
            ..Default::default()
        },
    );

    if menu.has_pending_reset_fields(active) {
        draw_text_ex(
            "Mode/Grid changes are pending. Press Enter on Mode or Grid to apply and reset run.",
            panel_x + 22.0,
            panel_y + panel_h - 50.0,
            TextParams {
                font_size: 21,
                color: theme.settings_pending,
                ..Default::default()
            },
        );
    }
}

fn high_score_for(game: &Game, high_scores: &HighScoreMap) -> u32 {
    let key = HighScoreKey::new(game.mode(), game.grid_width(), game.grid_height());
    high_scores.get(&key).copied().unwrap_or(0)
}

fn persist_state(settings: &Settings, high_scores: &HighScoreMap) {
    if let Err(err) = save_persisted(settings, high_scores) {
        eprintln!("warning: failed to persist settings/high scores: {err}");
    }
}

fn close_settings_menu(
    menu: &mut SettingsMenu,
    session_state: &mut SessionState,
    audio: &AudioManager,
    settings: &Settings,
) {
    let target = menu.close();
    *session_state = match target {
        MenuReturnState::Running => {
            audio.play_resume(settings.audio_enabled, settings.sfx_volume);
            SessionState::ResumeCountdown {
                remaining: RESUME_COUNTDOWN_SECONDS,
            }
        }
        MenuReturnState::Paused => SessionState::Paused,
        MenuReturnState::GameOver => SessionState::GameOver,
    };
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: 1024,
        window_height: 780,
        window_resizable: true,
        platform: macroquad::miniquad::conf::Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let (loaded_settings, mut high_scores) = load_persisted();
    let mut settings = loaded_settings.sanitized();
    let mut game = Game::new(settings.to_game_config(), 0.0);
    let mut theme_mode = settings.theme_default;
    let mut settings_menu = SettingsMenu::new(&settings);
    let mut session_state = SessionState::Running;
    let mut game_clock = 0.0f64;
    let mut accumulator = 0.0f32;
    let mut performance = PerformanceOverlay::new(get_time());
    let mut audio = AudioManager::load("assets/audio").await;
    #[cfg(not(target_arch = "wasm32"))]
    let mut frame_deadline = std::time::Instant::now();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            if settings_menu.open {
                close_settings_menu(&mut settings_menu, &mut session_state, &audio, &settings);
            } else {
                break;
            }
        }
        if is_key_pressed(KeyCode::Q) && !settings_menu.open {
            break;
        }

        if is_key_pressed(KeyCode::F) {
            performance.toggle_fps();
        }

        if is_key_pressed(KeyCode::Tab) {
            if settings_menu.open {
                close_settings_menu(&mut settings_menu, &mut session_state, &audio, &settings);
            } else {
                match session_state {
                    SessionState::Running => {
                        session_state = SessionState::Paused;
                        audio.play_pause(settings.audio_enabled, settings.sfx_volume);
                        settings_menu.open(&settings, MenuReturnState::Running);
                    }
                    SessionState::Paused => settings_menu.open(&settings, MenuReturnState::Paused),
                    SessionState::GameOver => {
                        settings_menu.open(&settings, MenuReturnState::GameOver)
                    }
                    SessionState::ResumeCountdown { .. } => {}
                }
            }
        }

        if settings_menu.open {
            if is_key_pressed(KeyCode::Up) {
                settings_menu.move_selection(-1);
            }
            if is_key_pressed(KeyCode::Down) {
                settings_menu.move_selection(1);
            }

            let mut changed = false;
            let field = settings_menu.selected_field();
            if is_key_pressed(KeyCode::Left) {
                changed = adjust_setting(
                    &mut settings_menu.draft,
                    field,
                    AdjustmentDirection::Decrease,
                );
            }
            if is_key_pressed(KeyCode::Right) {
                changed = adjust_setting(
                    &mut settings_menu.draft,
                    field,
                    AdjustmentDirection::Increase,
                );
            }

            if changed && !field.requires_enter_apply() {
                settings = settings_menu.draft.clone().sanitized();
                theme_mode = settings.theme_default;
                persist_state(&settings, &high_scores);
            }

            if is_key_pressed(KeyCode::Enter) && field.requires_enter_apply() {
                let mut reset_required = false;
                if settings.mode != settings_menu.draft.mode {
                    settings.mode = settings_menu.draft.mode;
                    reset_required = true;
                }
                if settings.grid_preset != settings_menu.draft.grid_preset {
                    settings.grid_preset = settings_menu.draft.grid_preset;
                    reset_required = true;
                }

                if reset_required {
                    settings = settings.sanitized();
                    settings_menu.draft = settings.clone();
                    settings_menu.return_state = MenuReturnState::Running;
                    game_clock = 0.0;
                    accumulator = 0.0;
                    game.set_config(settings.to_game_config(), game_clock);
                    session_state = SessionState::Paused;
                    persist_state(&settings, &high_scores);
                }
            }
        }

        if !settings_menu.open && is_key_pressed(KeyCode::M) {
            theme_mode.toggle();
            settings.theme_default = theme_mode;
            settings_menu.draft.theme_default = theme_mode;
            persist_state(&settings, &high_scores);
        }

        if !settings_menu.open && is_key_pressed(KeyCode::P) {
            match session_state {
                SessionState::Running => {
                    session_state = SessionState::Paused;
                    audio.play_pause(settings.audio_enabled, settings.sfx_volume);
                }
                SessionState::Paused => {
                    session_state = SessionState::ResumeCountdown {
                        remaining: RESUME_COUNTDOWN_SECONDS,
                    };
                    audio.play_resume(settings.audio_enabled, settings.sfx_volume);
                }
                SessionState::ResumeCountdown { .. } | SessionState::GameOver => {}
            }
        }

        if !settings_menu.open
            && is_key_pressed(KeyCode::R)
            && matches!(session_state, SessionState::GameOver)
        {
            game_clock = 0.0;
            accumulator = 0.0;
            game.reset(game_clock);
            session_state = SessionState::Running;
        }

        if !settings_menu.open
            && (matches!(session_state, SessionState::Running)
                || matches!(session_state, SessionState::ResumeCountdown { .. }))
        {
            handle_direction_input(&mut game);
        }

        let frame_time = get_frame_time();
        performance.tick(get_time());

        if matches!(session_state, SessionState::Running) {
            game_clock += frame_time as f64;
            accumulator += frame_time;

            while matches!(session_state, SessionState::Running) {
                let step = game.effective_step_seconds();
                if accumulator < step {
                    break;
                }

                let outcome = game.tick(game_clock);
                accumulator -= step;

                if outcome.ate_food {
                    audio.play_eat(settings.audio_enabled, settings.sfx_volume);
                }

                if outcome.died || outcome.won || game.is_over() {
                    session_state = SessionState::GameOver;
                    audio.play_death(settings.audio_enabled, settings.sfx_volume);

                    let key = HighScoreKey::new(game.mode(), game.grid_width(), game.grid_height());
                    let current_best = high_scores.get(&key).copied().unwrap_or(0);
                    if game.score() > current_best {
                        high_scores.insert(key, game.score());
                        persist_state(&settings, &high_scores);
                    }
                    break;
                }
            }
        }

        if let SessionState::ResumeCountdown { remaining } = &mut session_state {
            *remaining -= frame_time;
            if *remaining <= 0.0 {
                session_state = SessionState::Running;
                accumulator = 0.0;
            }
        }

        let step_progress = if matches!(session_state, SessionState::Running) {
            (accumulator / game.effective_step_seconds()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let layout = board_layout(game.grid_width(), game.grid_height());
        let colors = theme(theme_mode);
        let high_score = high_score_for(&game, &high_scores);

        let duck_bgm = settings_menu.open
            || matches!(session_state, SessionState::Paused)
            || matches!(session_state, SessionState::ResumeCountdown { .. });
        audio.update_bgm(settings.audio_enabled, settings.music_volume, duck_bgm);

        draw_background(&colors);
        draw_board(&layout, &colors, game.grid_width(), game.grid_height());
        draw_hud(
            &game,
            &layout,
            &performance,
            theme_mode,
            &colors,
            high_score,
        );
        draw_game(&game, &layout, step_progress, &colors);

        match session_state {
            SessionState::Paused => draw_pause_overlay(&layout, &colors),
            SessionState::ResumeCountdown { remaining } => {
                draw_resume_overlay(&layout, &colors, remaining)
            }
            SessionState::GameOver => draw_game_over_overlay(&game, &layout, &colors, high_score),
            SessionState::Running => {}
        }

        if settings_menu.open {
            draw_settings_overlay(&layout, &colors, &settings_menu, &settings);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            frame_deadline += std::time::Duration::from_secs_f64(TARGET_FRAME_SECONDS);
            let now = std::time::Instant::now();
            if frame_deadline > now {
                std::thread::sleep(frame_deadline - now);
            } else {
                frame_deadline = now;
            }
        }

        next_frame().await;
    }
}
