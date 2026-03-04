use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use ::rand::Rng;
use macroquad::prelude::*;

const GRID_WIDTH: i32 = 32;
const GRID_HEIGHT: i32 = 22;
const SNAKE_STEP_SECONDS: f32 = 0.11;
const PERF_SAMPLE_SECONDS: f32 = 1.0;
const DEFAULT_FPS_CAP: f64 = 240.0;
const FRAME_BUDGET_SECONDS: f64 = 1.0 / DEFAULT_FPS_CAP;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }
}

struct Game {
    grid_width: i32,
    grid_height: i32,
    snake: VecDeque<Point>,
    dir: Direction,
    queued_dirs: VecDeque<Direction>,
    food: Point,
    score: u32,
    over: bool,
    won: bool,
}

impl Game {
    fn new(grid_width: i32, grid_height: i32) -> Self {
        let mut game = Self {
            grid_width,
            grid_height,
            snake: VecDeque::new(),
            dir: Direction::Right,
            queued_dirs: VecDeque::new(),
            food: Point { x: 0, y: 0 },
            score: 0,
            over: false,
            won: false,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.snake.clear();
        let start_x = self.grid_width / 2;
        let start_y = self.grid_height / 2;
        self.snake.push_back(Point {
            x: start_x,
            y: start_y,
        });
        self.snake.push_back(Point {
            x: start_x - 1,
            y: start_y,
        });
        self.snake.push_back(Point {
            x: start_x - 2,
            y: start_y,
        });

        self.dir = Direction::Right;
        self.queued_dirs.clear();
        self.score = 0;
        self.over = false;
        self.won = false;
        self.food = self.spawn_food();
    }

    fn set_direction(&mut self, direction: Direction) {
        let reference = self.queued_dirs.back().copied().unwrap_or(self.dir);
        if direction == reference || direction.is_opposite(reference) {
            return;
        }

        // Keep a short buffer so quick turn combos are not dropped between ticks.
        if self.queued_dirs.len() < 2 {
            self.queued_dirs.push_back(direction);
        }
    }

    fn preview_direction(&self) -> Direction {
        self.queued_dirs.front().copied().unwrap_or(self.dir)
    }

    fn tick(&mut self) {
        if self.over {
            return;
        }

        if let Some(queued) = self.queued_dirs.pop_front() {
            self.dir = queued;
        }
        let current_head = *self.snake.front().expect("snake always has a head");
        let (dx, dy) = self.dir.delta();
        let new_head = Point {
            x: current_head.x + dx,
            y: current_head.y + dy,
        };

        if new_head.x < 0
            || new_head.y < 0
            || new_head.x >= self.grid_width
            || new_head.y >= self.grid_height
        {
            self.over = true;
            return;
        }

        let growing = new_head == self.food;
        let tail = *self.snake.back().expect("snake always has a tail");
        let collided = self.snake.iter().any(|&segment| {
            if segment != new_head {
                return false;
            }
            if !growing && segment == tail {
                return false;
            }
            true
        });

        if collided {
            self.over = true;
            return;
        }

        self.snake.push_front(new_head);
        if growing {
            self.score += 1;
            if self.snake.len() == (self.grid_width * self.grid_height) as usize {
                self.over = true;
                self.won = true;
                return;
            }
            self.food = self.spawn_food();
        } else {
            let _ = self.snake.pop_back();
        }
    }

    fn spawn_food(&self) -> Point {
        let mut rng = ::rand::thread_rng();
        loop {
            let point = Point {
                x: rng.gen_range(0..self.grid_width),
                y: rng.gen_range(0..self.grid_height),
            };
            if !self.snake.contains(&point) {
                return point;
            }
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
    frame_ms_display: f32,
}

impl PerformanceOverlay {
    fn new(now: f64) -> Self {
        Self {
            show_fps: true,
            sample_start_time: now,
            sample_frames: 0,
            fps_display: 0.0,
            frame_ms_display: 0.0,
        }
    }

    fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
    }

    fn tick(&mut self, now: f64) {
        self.sample_frames += 1;
        let sample_elapsed = (now - self.sample_start_time) as f32;

        // FRAPS/Afterburner-style: count completed frames over real elapsed wall time.
        if sample_elapsed >= PERF_SAMPLE_SECONDS {
            let frames = self.sample_frames.max(1) as f32;
            self.fps_display = frames / sample_elapsed;
            self.frame_ms_display = (sample_elapsed * 1000.0) / frames;
            self.sample_start_time = now;
            self.sample_frames = 0;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        };
    }

    fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
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
        },
    }
}

fn board_layout() -> BoardLayout {
    let sw = screen_width();
    let sh = screen_height();
    let side_margin = 84.0;
    let top_space = 150.0;
    let bottom_space = 88.0;

    let cell_x = (sw - side_margin * 2.0) / GRID_WIDTH as f32;
    let cell_y = (sh - top_space - bottom_space) / GRID_HEIGHT as f32;
    let cell = cell_x.min(cell_y).max(8.0);
    let width_px = cell * GRID_WIDTH as f32;
    let height_px = cell * GRID_HEIGHT as f32;
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

fn handle_input(game: &mut Game, theme_mode: &mut ThemeMode, performance: &mut PerformanceOverlay) {
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
    if is_key_pressed(KeyCode::R) && game.over {
        game.reset();
    }
    if is_key_pressed(KeyCode::M) {
        theme_mode.toggle();
    }
    if is_key_pressed(KeyCode::F) {
        performance.toggle_fps();
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

fn draw_board(layout: &BoardLayout, theme: &Theme) {
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

    for x in 1..GRID_WIDTH {
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
    for y in 1..GRID_HEIGHT {
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

fn next_move_would_collide(game: &Game) -> bool {
    if game.over {
        return false;
    }

    let current_head = *game.snake.front().expect("snake always has a head");
    let (dx, dy) = game.preview_direction().delta();
    let new_head = Point {
        x: current_head.x + dx,
        y: current_head.y + dy,
    };

    if new_head.x < 0
        || new_head.y < 0
        || new_head.x >= game.grid_width
        || new_head.y >= game.grid_height
    {
        return true;
    }

    let growing = new_head == game.food;
    let tail = *game.snake.back().expect("snake always has a tail");
    game.snake.iter().any(|&segment| {
        if segment != new_head {
            return false;
        }
        if !growing && segment == tail {
            return false;
        }
        true
    })
}

fn draw_hud(
    game: &Game,
    layout: &BoardLayout,
    performance: &PerformanceOverlay,
    theme_mode: ThemeMode,
    theme: &Theme,
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

    let info = if performance.show_fps {
        format!(
            "Score: {}    FPS: {:.0} ({:.1} ms)    Theme: {}",
            game.score,
            performance.fps_display,
            performance.frame_ms_display,
            theme_mode.label()
        )
    } else {
        format!("Score: {}    Theme: {}", game.score, theme_mode.label())
    };
    draw_text_ex(
        &info,
        layout.x,
        layout.y - 18.0,
        TextParams {
            font_size: 30,
            color: theme.hud_text,
            ..Default::default()
        },
    );

    draw_text_ex(
        "Move: arrows or WASD  |  Restart: R  |  Theme: M  |  FPS: F  |  Quit: Q / Esc",
        layout.x,
        layout.y + layout.height_px + 42.0,
        TextParams {
            font_size: 24,
            color: theme.controls_text,
            ..Default::default()
        },
    );
}

fn draw_game(game: &Game, layout: &BoardLayout, step_progress: f32, theme: &Theme) {
    let food_center_x = layout.x + game.food.x as f32 * layout.cell + layout.cell * 0.5;
    let food_center_y = layout.y + game.food.y as f32 * layout.cell + layout.cell * 0.5;
    let pulse = (get_time() as f32 * 4.0).sin() * 0.06 + 1.0;
    let food_radius = layout.cell * 0.27 * pulse;
    draw_circle(
        food_center_x,
        food_center_y,
        food_radius + 2.0,
        theme.food_glow,
    );
    draw_circle(food_center_x, food_center_y, food_radius, theme.food);

    let snake_len = game.snake.len();
    let max_idx = (snake_len.saturating_sub(1)).max(1) as f32;
    let safe_progress = if next_move_would_collide(game) {
        0.0
    } else {
        step_progress
    };

    for idx in (1..snake_len).rev() {
        let segment = game.snake[idx];
        let target = game.snake[idx - 1];
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

    if let Some(head) = game.snake.front().copied() {
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

fn draw_overlay(game: &Game, layout: &BoardLayout, theme: &Theme) {
    if !game.over {
        return;
    }

    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        theme.overlay,
    );

    let headline = if game.won { "You Win" } else { "Game Over" };
    let headline_dim = measure_text(headline, None, 64, 1.0);
    draw_text_ex(
        headline,
        layout.x + (layout.width_px - headline_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 - 12.0,
        TextParams {
            font_size: 64,
            color: theme.overlay_title,
            ..Default::default()
        },
    );

    let sub = "Press R to restart";
    let sub_dim = measure_text(sub, None, 32, 1.0);
    draw_text_ex(
        sub,
        layout.x + (layout.width_px - sub_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 + 32.0,
        TextParams {
            font_size: 32,
            color: theme.overlay_subtitle,
            ..Default::default()
        },
    );
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: 1024,
        window_height: 780,
        window_resizable: true,
        platform: macroquad::miniquad::conf::Platform {
            // Disable vsync for uncapped rendering throughput.
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(GRID_WIDTH, GRID_HEIGHT);
    let mut accumulator = 0.0f32;
    let mut theme_mode = ThemeMode::Light;
    let mut performance = PerformanceOverlay::new(get_time());

    loop {
        let frame_start = get_time();

        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape) {
            break;
        }

        handle_input(&mut game, &mut theme_mode, &mut performance);
        let frame_time = get_frame_time();
        accumulator += frame_time;
        while accumulator >= SNAKE_STEP_SECONDS {
            game.tick();
            accumulator -= SNAKE_STEP_SECONDS;
        }

        performance.tick(get_time());
        let step_progress = if game.over {
            0.0
        } else {
            (accumulator / SNAKE_STEP_SECONDS).clamp(0.0, 1.0)
        };

        let layout = board_layout();
        let colors = theme(theme_mode);
        draw_background(&colors);
        draw_board(&layout, &colors);
        draw_hud(&game, &layout, &performance, theme_mode, &colors);
        draw_game(&game, &layout, step_progress, &colors);
        draw_overlay(&game, &layout, &colors);

        let frame_elapsed = get_time() - frame_start;
        if frame_elapsed < FRAME_BUDGET_SECONDS {
            thread::sleep(Duration::from_secs_f64(
                FRAME_BUDGET_SECONDS - frame_elapsed,
            ));
        }

        next_frame().await;
    }
}
