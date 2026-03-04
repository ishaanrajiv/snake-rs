use std::collections::VecDeque;

use ::rand::Rng;
use serde::{Deserialize, Serialize};

pub const COMBO_WINDOW_SECONDS: f64 = 2.5;
pub const COMBO_MAX_MULTIPLIER: u32 = 5;
pub const DIFFICULTY_RAMP_FACTOR: f32 = 0.985;
pub const MIN_STEP_SECONDS: f32 = 0.055;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Wrap,
    Zen,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Wrap => "Wrap",
            Self::Zen => "Zen",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Classic => Self::Wrap,
            Self::Wrap => Self::Zen,
            Self::Zen => Self::Classic,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Classic => Self::Zen,
            Self::Wrap => Self::Classic,
            Self::Zen => Self::Wrap,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HighScoreKey {
    pub mode: GameMode,
    pub grid_width: i32,
    pub grid_height: i32,
}

impl HighScoreKey {
    pub fn new(mode: GameMode, grid_width: i32, grid_height: i32) -> Self {
        Self {
            mode,
            grid_width,
            grid_height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GameConfig {
    pub grid_width: i32,
    pub grid_height: i32,
    pub mode: GameMode,
    pub base_step_seconds: f32,
    pub difficulty_enabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ComboState {
    pub current_multiplier: u32,
    pub max_multiplier: u32,
    pub last_food_time: Option<f64>,
}

impl ComboState {
    fn new() -> Self {
        Self {
            current_multiplier: 0,
            max_multiplier: 0,
            last_food_time: None,
        }
    }

    fn register_food(&mut self, now: f64) -> u32 {
        let next = match self.last_food_time {
            Some(last) if now - last <= COMBO_WINDOW_SECONDS => self.current_multiplier + 1,
            _ => 1,
        };
        self.current_multiplier = next.min(COMBO_MAX_MULTIPLIER);
        self.max_multiplier = self.max_multiplier.max(self.current_multiplier);
        self.last_food_time = Some(now);
        self.current_multiplier
    }
}

#[derive(Clone, Debug)]
pub struct RunStats {
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub foods_eaten: u32,
    pub max_combo: u32,
    pub final_score: u32,
    pub avg_foods_per_minute: f32,
}

impl RunStats {
    fn new(now: f64) -> Self {
        Self {
            start_time: now,
            end_time: None,
            foods_eaten: 0,
            max_combo: 0,
            final_score: 0,
            avg_foods_per_minute: 0.0,
        }
    }

    fn record_food(&mut self, combo_multiplier: u32) {
        self.foods_eaten += 1;
        self.max_combo = self.max_combo.max(combo_multiplier);
    }

    fn finalize(&mut self, now: f64, score: u32) {
        self.end_time = Some(now);
        self.final_score = score;
        let elapsed_minutes = ((now - self.start_time) as f32 / 60.0).max(0.0001);
        self.avg_foods_per_minute = self.foods_eaten as f32 / elapsed_minutes;
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.end_time
            .map(|end| (end - self.start_time) as f32)
            .unwrap_or(0.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TickOutcome {
    pub ate_food: bool,
    pub died: bool,
    pub won: bool,
    pub points_gained: u32,
    pub combo_multiplier: u32,
}

pub struct Game {
    config: GameConfig,
    snake: VecDeque<Point>,
    dir: Direction,
    queued_dirs: VecDeque<Direction>,
    food: Point,
    score: u32,
    over: bool,
    won: bool,
    combo: ComboState,
    run_stats: RunStats,
}

impl Game {
    pub fn new(config: GameConfig, now: f64) -> Self {
        let mut game = Self {
            config,
            snake: VecDeque::new(),
            dir: Direction::Right,
            queued_dirs: VecDeque::new(),
            food: Point { x: 0, y: 0 },
            score: 0,
            over: false,
            won: false,
            combo: ComboState::new(),
            run_stats: RunStats::new(now),
        };
        game.reset(now);
        game
    }

    pub fn reset(&mut self, now: f64) {
        self.snake.clear();
        let start_x = self.config.grid_width / 2;
        let start_y = self.config.grid_height / 2;
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
        self.combo = ComboState::new();
        self.run_stats = RunStats::new(now);
        self.food = self.spawn_food();
    }

    pub fn set_config(&mut self, config: GameConfig, now: f64) {
        self.config = config;
        self.reset(now);
    }

    pub fn set_direction(&mut self, direction: Direction) {
        let reference = self.queued_dirs.back().copied().unwrap_or(self.dir);
        if direction == reference || direction.is_opposite(reference) {
            return;
        }

        if self.queued_dirs.len() < 2 {
            self.queued_dirs.push_back(direction);
        }
    }

    pub fn preview_direction(&self) -> Direction {
        self.queued_dirs.front().copied().unwrap_or(self.dir)
    }

    fn wrap_axis(value: i32, limit: i32) -> i32 {
        value.rem_euclid(limit)
    }

    fn mode_adjusted_head(&self, mut point: Point) -> Option<Point> {
        match self.config.mode {
            GameMode::Classic => {
                if point.x < 0
                    || point.y < 0
                    || point.x >= self.config.grid_width
                    || point.y >= self.config.grid_height
                {
                    None
                } else {
                    Some(point)
                }
            }
            GameMode::Wrap | GameMode::Zen => {
                point.x = Self::wrap_axis(point.x, self.config.grid_width);
                point.y = Self::wrap_axis(point.y, self.config.grid_height);
                Some(point)
            }
        }
    }

    fn would_hit_body(&self, new_head: Point, growing: bool) -> bool {
        if self.config.mode == GameMode::Zen {
            return false;
        }

        let tail = *self.snake.back().expect("snake always has a tail");
        self.snake.iter().any(|&segment| {
            if segment != new_head {
                return false;
            }
            if !growing && segment == tail {
                return false;
            }
            true
        })
    }

    fn end_run(&mut self, now: f64, won: bool) -> TickOutcome {
        self.over = true;
        self.won = won;
        self.run_stats.finalize(now, self.score);
        TickOutcome {
            died: !won,
            won,
            ..Default::default()
        }
    }

    pub fn tick(&mut self, now: f64) -> TickOutcome {
        if self.over {
            return TickOutcome::default();
        }

        if let Some(queued) = self.queued_dirs.pop_front() {
            self.dir = queued;
        }

        let current_head = *self.snake.front().expect("snake always has a head");
        let (dx, dy) = self.dir.delta();
        let proposed_head = Point {
            x: current_head.x + dx,
            y: current_head.y + dy,
        };

        let Some(new_head) = self.mode_adjusted_head(proposed_head) else {
            return self.end_run(now, false);
        };

        let growing = new_head == self.food;
        if self.would_hit_body(new_head, growing) {
            return self.end_run(now, false);
        }

        self.snake.push_front(new_head);

        if growing {
            let combo_multiplier = self.combo.register_food(now);
            self.run_stats.record_food(combo_multiplier);
            self.score += combo_multiplier;

            if self.snake.len() == (self.config.grid_width * self.config.grid_height) as usize {
                let mut outcome = self.end_run(now, true);
                outcome.ate_food = true;
                outcome.points_gained = combo_multiplier;
                outcome.combo_multiplier = combo_multiplier;
                return outcome;
            }

            self.food = self.spawn_food();
            return TickOutcome {
                ate_food: true,
                points_gained: combo_multiplier,
                combo_multiplier,
                ..Default::default()
            };
        }

        let _ = self.snake.pop_back();
        TickOutcome::default()
    }

    fn spawn_food(&self) -> Point {
        let mut rng = ::rand::thread_rng();
        loop {
            let point = Point {
                x: rng.gen_range(0..self.config.grid_width),
                y: rng.gen_range(0..self.config.grid_height),
            };
            if !self.snake.contains(&point) {
                return point;
            }
        }
    }

    pub fn next_move_would_collide(&self) -> bool {
        if self.over || self.config.mode == GameMode::Zen {
            return false;
        }

        let current_head = *self.snake.front().expect("snake always has a head");
        let (dx, dy) = self.preview_direction().delta();
        let proposed = Point {
            x: current_head.x + dx,
            y: current_head.y + dy,
        };
        let Some(new_head) = self.mode_adjusted_head(proposed) else {
            return true;
        };

        let growing = new_head == self.food;
        self.would_hit_body(new_head, growing)
    }

    pub fn effective_step_seconds(&self) -> f32 {
        if !self.config.difficulty_enabled {
            return self.config.base_step_seconds;
        }
        let score = i32::try_from(self.score).unwrap_or(i32::MAX);
        let scaled = self.config.base_step_seconds * DIFFICULTY_RAMP_FACTOR.powi(score);
        scaled.max(MIN_STEP_SECONDS)
    }

    pub fn snake(&self) -> &VecDeque<Point> {
        &self.snake
    }

    pub fn food(&self) -> Point {
        self.food
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn is_over(&self) -> bool {
        self.over
    }

    pub fn is_won(&self) -> bool {
        self.won
    }

    pub fn combo_state(&self) -> ComboState {
        self.combo
    }

    pub fn run_stats(&self) -> &RunStats {
        &self.run_stats
    }

    pub fn mode(&self) -> GameMode {
        self.config.mode
    }

    pub fn grid_width(&self) -> i32 {
        self.config.grid_width
    }

    pub fn grid_height(&self) -> i32 {
        self.config.grid_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cfg(mode: GameMode, width: i32, height: i32) -> GameConfig {
        GameConfig {
            grid_width: width,
            grid_height: height,
            mode,
            base_step_seconds: 0.11,
            difficulty_enabled: true,
        }
    }

    #[test]
    fn classic_wall_collision_ends_run() {
        let mut game = Game::new(cfg(GameMode::Classic, 4, 4), 0.0);
        let _ = game.tick(0.2);
        let out = game.tick(0.4);
        assert!(game.is_over());
        assert!(out.died);
    }

    #[test]
    fn wrap_mode_crosses_edge_without_death() {
        let mut game = Game::new(cfg(GameMode::Wrap, 4, 4), 0.0);
        let _ = game.tick(0.2);
        let out = game.tick(0.4);
        let head = game.snake().front().copied().expect("head exists");
        assert_eq!(head.x, 0);
        assert!(!game.is_over());
        assert!(!out.died);
    }

    #[test]
    fn zen_mode_allows_self_overlap() {
        let mut game = Game::new(cfg(GameMode::Zen, 5, 5), 0.0);
        game.snake = VecDeque::from(vec![
            Point { x: 1, y: 1 },
            Point { x: 1, y: 2 },
            Point { x: 2, y: 2 },
            Point { x: 2, y: 1 },
        ]);
        game.dir = Direction::Down;
        let _ = game.tick(0.5);
        assert!(!game.is_over());
    }

    #[test]
    fn combo_increments_and_resets_by_time_window() {
        let mut game = Game::new(cfg(GameMode::Classic, 10, 10), 0.0);

        let head = *game.snake().front().expect("head exists");
        game.food = Point {
            x: head.x + 1,
            y: head.y,
        };
        let first = game.tick(0.5);

        let head = *game.snake().front().expect("head exists");
        game.food = Point {
            x: head.x + 1,
            y: head.y,
        };
        let second = game.tick(2.0);

        let head = *game.snake().front().expect("head exists");
        game.food = Point {
            x: head.x + 1,
            y: head.y,
        };
        let third = game.tick(5.5);

        assert_eq!(first.combo_multiplier, 1);
        assert_eq!(second.combo_multiplier, 2);
        assert_eq!(third.combo_multiplier, 1);
    }

    #[test]
    fn combo_multiplier_is_capped_at_five() {
        let mut combo = ComboState::new();
        let mut value = 0;
        for i in 0..10 {
            value = combo.register_food(1.0 + i as f64 * 0.2);
        }
        assert_eq!(value, 5);
        assert_eq!(combo.max_multiplier, 5);
    }

    #[test]
    fn difficulty_curve_is_monotonic_and_has_floor() {
        let mut game = Game::new(cfg(GameMode::Classic, 12, 12), 0.0);
        let mut previous = game.effective_step_seconds();
        for score in 1..500 {
            game.score = score;
            let current = game.effective_step_seconds();
            assert!(current <= previous + f32::EPSILON);
            assert!(current >= MIN_STEP_SECONDS - f32::EPSILON);
            previous = current;
        }
    }

    #[test]
    fn high_score_keys_are_separate_by_mode_and_grid() {
        let mut map: HashMap<HighScoreKey, u32> = HashMap::new();
        map.insert(HighScoreKey::new(GameMode::Classic, 32, 22), 10);
        map.insert(HighScoreKey::new(GameMode::Wrap, 32, 22), 20);
        map.insert(HighScoreKey::new(GameMode::Classic, 40, 28), 30);

        assert_eq!(map.len(), 3);
        assert_eq!(
            map.get(&HighScoreKey::new(GameMode::Classic, 32, 22)),
            Some(&10)
        );
        assert_eq!(
            map.get(&HighScoreKey::new(GameMode::Wrap, 32, 22)),
            Some(&20)
        );
        assert_eq!(
            map.get(&HighScoreKey::new(GameMode::Classic, 40, 28)),
            Some(&30)
        );
    }
}
