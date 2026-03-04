use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::game::{GameMode, HighScoreKey};
use crate::settings::Settings;

pub type HighScoreMap = HashMap<HighScoreKey, u32>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedHighScore {
    mode: GameMode,
    grid_width: i32,
    grid_height: i32,
    score: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedData {
    pub settings: Settings,
    high_scores: Vec<PersistedHighScore>,
}

impl Default for PersistedData {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            high_scores: Vec::new(),
        }
    }
}

pub fn load_persisted() -> (Settings, HighScoreMap) {
    load_from_path(&default_data_path())
}

pub fn save_persisted(settings: &Settings, high_scores: &HighScoreMap) -> io::Result<()> {
    save_to_path(&default_data_path(), settings, high_scores)
}

fn default_data_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("snake-rs").join("save.json")
}

fn map_to_scores(map: &HighScoreMap) -> Vec<PersistedHighScore> {
    let mut vec: Vec<PersistedHighScore> = map
        .iter()
        .map(|(key, score)| PersistedHighScore {
            mode: key.mode,
            grid_width: key.grid_width,
            grid_height: key.grid_height,
            score: *score,
        })
        .collect();
    vec.sort_by_key(|entry| {
        (
            mode_sort_key(entry.mode),
            entry.grid_width,
            entry.grid_height,
            entry.score,
        )
    });
    vec
}

fn scores_to_map(scores: &[PersistedHighScore]) -> HighScoreMap {
    let mut map = HighScoreMap::new();
    for entry in scores {
        let key = HighScoreKey::new(entry.mode, entry.grid_width, entry.grid_height);
        let current = map.get(&key).copied().unwrap_or(0);
        if entry.score > current {
            map.insert(key, entry.score);
        }
    }
    map
}

fn mode_sort_key(mode: GameMode) -> i32 {
    match mode {
        GameMode::Classic => 0,
        GameMode::Wrap => 1,
        GameMode::Zen => 2,
    }
}

fn load_from_path(path: &Path) -> (Settings, HighScoreMap) {
    if !path.exists() {
        let defaults = PersistedData::default();
        return (defaults.settings, HighScoreMap::new());
    }

    let content = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "warning: failed to read persisted data at {}: {}",
                path.display(),
                err
            );
            return (Settings::default(), HighScoreMap::new());
        }
    };

    match serde_json::from_str::<PersistedData>(&content) {
        Ok(data) => (data.settings.sanitized(), scores_to_map(&data.high_scores)),
        Err(err) => {
            eprintln!(
                "warning: invalid persisted data at {}: {}; using defaults",
                path.display(),
                err
            );
            (Settings::default(), HighScoreMap::new())
        }
    }
}

fn save_to_path(path: &Path, settings: &Settings, high_scores: &HighScoreMap) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = PersistedData {
        settings: settings.clone().sanitized(),
        high_scores: map_to_scores(high_scores),
    };
    let json = serde_json::to_string_pretty(&data)
        .map_err(|err| io::Error::other(format!("serialize failure: {err}")))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join("snake-rs-tests")
            .join(format!("{label}-{nanos}.json"))
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let path = test_path("missing");
        let (settings, high_scores) = load_from_path(&path);
        assert_eq!(settings.mode, Settings::default().mode);
        assert!(high_scores.is_empty());
    }

    #[test]
    fn persisted_round_trip_works() {
        let path = test_path("round-trip");
        let mut map = HighScoreMap::new();
        map.insert(HighScoreKey::new(GameMode::Classic, 32, 22), 42);
        let settings = Settings::default();

        save_to_path(&path, &settings, &map).expect("save succeeds");
        let (loaded_settings, loaded_map) = load_from_path(&path);

        assert_eq!(loaded_settings.mode, settings.mode);
        assert_eq!(loaded_map.len(), 1);
        assert_eq!(
            loaded_map.get(&HighScoreKey::new(GameMode::Classic, 32, 22)),
            Some(&42)
        );
    }

    #[test]
    fn corrupt_json_falls_back_to_defaults() {
        let path = test_path("corrupt");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(&path, "{ definitely-not-json ").expect("write corrupt content");

        let (settings, high_scores) = load_from_path(&path);
        assert_eq!(settings.mode, Settings::default().mode);
        assert!(high_scores.is_empty());
    }
}
