use std::collections::VecDeque;

use ::rand::Rng;
use macroquad::prelude::*;

const GRID_WIDTH: i32 = 32;
const GRID_HEIGHT: i32 = 22;
const STEP_SECONDS: f32 = 0.11;

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
    next_dir: Direction,
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
            next_dir: Direction::Right,
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
        self.next_dir = Direction::Right;
        self.score = 0;
        self.over = false;
        self.won = false;
        self.food = self.spawn_food();
    }

    fn set_direction(&mut self, direction: Direction) {
        if !self.next_dir.is_opposite(direction) && !self.dir.is_opposite(direction) {
            self.next_dir = direction;
        }
    }

    fn tick(&mut self) {
        if self.over {
            return;
        }

        self.dir = self.next_dir;
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

fn handle_input(game: &mut Game) {
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
}

fn draw_background() {
    clear_background(pastel(248, 246, 255));
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;

    draw_circle(
        sw * 0.1 + t.sin() * 14.0,
        sh * 0.2,
        sh * 0.28,
        pastel_alpha(255, 212, 226, 0.34),
    );
    draw_circle(
        sw * 0.86 + t.cos() * 18.0,
        sh * 0.24,
        sh * 0.24,
        pastel_alpha(206, 234, 255, 0.32),
    );
    draw_circle(
        sw * 0.7 + (t * 0.7).sin() * 11.0,
        sh * 0.84,
        sh * 0.34,
        pastel_alpha(216, 244, 221, 0.28),
    );
}

fn draw_board(layout: &BoardLayout) {
    draw_rectangle(
        layout.x - 16.0,
        layout.y - 16.0,
        layout.width_px + 32.0,
        layout.height_px + 32.0,
        pastel_alpha(255, 255, 255, 0.72),
    );
    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        pastel(255, 252, 250),
    );
    draw_rectangle_lines(
        layout.x - 1.5,
        layout.y - 1.5,
        layout.width_px + 3.0,
        layout.height_px + 3.0,
        3.0,
        pastel(206, 199, 235),
    );

    for x in 1..GRID_WIDTH {
        let px = layout.x + x as f32 * layout.cell;
        draw_line(
            px,
            layout.y,
            px,
            layout.y + layout.height_px,
            1.0,
            pastel_alpha(213, 214, 226, 0.45),
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
            pastel_alpha(213, 214, 226, 0.45),
        );
    }
}

fn draw_hud(game: &Game, layout: &BoardLayout) {
    let title = "Pastel Snake";
    let title_dim = measure_text(title, None, 54, 1.0);
    draw_text_ex(
        title,
        (screen_width() - title_dim.width) * 0.5,
        layout.y - 62.0,
        TextParams {
            font_size: 54,
            color: pastel(126, 112, 170),
            ..Default::default()
        },
    );

    let info = format!("Score: {}", game.score);
    draw_text_ex(
        &info,
        layout.x,
        layout.y - 18.0,
        TextParams {
            font_size: 30,
            color: pastel(111, 129, 166),
            ..Default::default()
        },
    );

    draw_text_ex(
        "Move: arrows or WASD  |  Restart: R  |  Quit: Q / Esc",
        layout.x,
        layout.y + layout.height_px + 42.0,
        TextParams {
            font_size: 24,
            color: pastel(134, 141, 160),
            ..Default::default()
        },
    );
}

fn draw_game(game: &Game, layout: &BoardLayout) {
    let food_center_x = layout.x + game.food.x as f32 * layout.cell + layout.cell * 0.5;
    let food_center_y = layout.y + game.food.y as f32 * layout.cell + layout.cell * 0.5;
    let pulse = (get_time() as f32 * 4.0).sin() * 0.06 + 1.0;
    let food_radius = layout.cell * 0.27 * pulse;
    draw_circle(
        food_center_x,
        food_center_y,
        food_radius + 2.0,
        pastel_alpha(255, 172, 162, 0.4),
    );
    draw_circle(
        food_center_x,
        food_center_y,
        food_radius,
        pastel(250, 144, 136),
    );

    let body_start = pastel(168, 226, 198);
    let body_end = pastel(188, 170, 232);
    let max_idx = (game.snake.len().saturating_sub(1)).max(1) as f32;
    for (idx, segment) in game.snake.iter().enumerate().skip(1).rev() {
        let t = idx as f32 / max_idx;
        let color = lerp_color(body_start, body_end, t);
        let inset = layout.cell * 0.12;
        let x = layout.x + segment.x as f32 * layout.cell + inset;
        let y = layout.y + segment.y as f32 * layout.cell + inset;
        let size = layout.cell - inset * 2.0;
        draw_rectangle(x, y, size, size, color);
    }

    if let Some(head) = game.snake.front() {
        let inset = layout.cell * 0.09;
        let x = layout.x + head.x as f32 * layout.cell + inset;
        let y = layout.y + head.y as f32 * layout.cell + inset;
        let size = layout.cell - inset * 2.0;
        draw_rectangle(x, y, size, size, pastel(113, 196, 173));

        let eye_offset = layout.cell * 0.18;
        let eye_radius = layout.cell * 0.05;
        let (ex1, ey1, ex2, ey2) = match game.dir {
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
        draw_circle(ex1, ey1, eye_radius, pastel(66, 89, 100));
        draw_circle(ex2, ey2, eye_radius, pastel(66, 89, 100));
    }
}

fn draw_overlay(game: &Game, layout: &BoardLayout) {
    if !game.over {
        return;
    }

    draw_rectangle(
        layout.x,
        layout.y,
        layout.width_px,
        layout.height_px,
        pastel_alpha(255, 255, 255, 0.62),
    );

    let headline = if game.won { "You Win" } else { "Game Over" };
    let headline_dim = measure_text(headline, None, 64, 1.0);
    draw_text_ex(
        headline,
        layout.x + (layout.width_px - headline_dim.width) * 0.5,
        layout.y + layout.height_px * 0.5 - 12.0,
        TextParams {
            font_size: 64,
            color: pastel(124, 108, 170),
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
            color: pastel(111, 129, 166),
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
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(GRID_WIDTH, GRID_HEIGHT);
    let mut accumulator = 0.0f32;

    loop {
        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape) {
            break;
        }

        handle_input(&mut game);
        accumulator += get_frame_time();
        while accumulator >= STEP_SECONDS {
            game.tick();
            accumulator -= STEP_SECONDS;
        }

        let layout = board_layout();
        draw_background();
        draw_board(&layout);
        draw_hud(&game, &layout);
        draw_game(&game, &layout);
        draw_overlay(&game, &layout);

        next_frame().await;
    }
}
