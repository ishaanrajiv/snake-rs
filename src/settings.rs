use serde::{Deserialize, Serialize};

use crate::game::{GameConfig, GameMode};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        };
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub fn previous(self) -> Self {
        self.next()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GridPreset {
    Small,
    Medium,
    Large,
}

impl GridPreset {
    pub fn dimensions(self) -> (i32, i32) {
        match self {
            Self::Small => (24, 16),
            Self::Medium => (32, 22),
            Self::Large => (40, 28),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small (24x16)",
            Self::Medium => "Medium (32x22)",
            Self::Large => "Large (40x28)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Large,
            Self::Large => Self::Small,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Small => Self::Large,
            Self::Medium => Self::Small,
            Self::Large => Self::Medium,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub mode: GameMode,
    pub grid_preset: GridPreset,
    pub base_speed: f32,
    pub difficulty_enabled: bool,
    pub theme_default: ThemeMode,
    pub audio_enabled: bool,
    pub music_volume: u8,
    pub sfx_volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: GameMode::Classic,
            grid_preset: GridPreset::Medium,
            base_speed: 0.11,
            difficulty_enabled: true,
            theme_default: ThemeMode::Light,
            audio_enabled: true,
            music_volume: 65,
            sfx_volume: 75,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        self.base_speed = self.base_speed.clamp(0.07, 0.20);
        self.music_volume = self.music_volume.min(100);
        self.sfx_volume = self.sfx_volume.min(100);
        self
    }

    pub fn to_game_config(&self) -> GameConfig {
        let (grid_width, grid_height) = self.grid_preset.dimensions();
        GameConfig {
            grid_width,
            grid_height,
            mode: self.mode,
            base_step_seconds: self.base_speed,
            difficulty_enabled: self.difficulty_enabled,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingField {
    Mode,
    GridPreset,
    BaseSpeed,
    Difficulty,
    Theme,
    AudioEnabled,
    MusicVolume,
    SfxVolume,
}

impl SettingField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mode => "Mode",
            Self::GridPreset => "Grid",
            Self::BaseSpeed => "Base Speed",
            Self::Difficulty => "Difficulty Ramp",
            Self::Theme => "Theme",
            Self::AudioEnabled => "Audio",
            Self::MusicVolume => "Music Vol",
            Self::SfxVolume => "SFX Vol",
        }
    }

    pub fn requires_enter_apply(self) -> bool {
        matches!(self, Self::Mode | Self::GridPreset)
    }
}

pub const ALL_SETTING_FIELDS: [SettingField; 8] = [
    SettingField::Mode,
    SettingField::GridPreset,
    SettingField::BaseSpeed,
    SettingField::Difficulty,
    SettingField::Theme,
    SettingField::AudioEnabled,
    SettingField::MusicVolume,
    SettingField::SfxVolume,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentDirection {
    Decrease,
    Increase,
}

pub fn adjust_setting(
    settings: &mut Settings,
    field: SettingField,
    direction: AdjustmentDirection,
) -> bool {
    let original = settings.clone();
    match field {
        SettingField::Mode => {
            settings.mode = match direction {
                AdjustmentDirection::Increase => settings.mode.next(),
                AdjustmentDirection::Decrease => settings.mode.previous(),
            };
        }
        SettingField::GridPreset => {
            settings.grid_preset = match direction {
                AdjustmentDirection::Increase => settings.grid_preset.next(),
                AdjustmentDirection::Decrease => settings.grid_preset.previous(),
            };
        }
        SettingField::BaseSpeed => {
            let step = 0.005;
            settings.base_speed = match direction {
                AdjustmentDirection::Increase => (settings.base_speed + step).clamp(0.07, 0.20),
                AdjustmentDirection::Decrease => (settings.base_speed - step).clamp(0.07, 0.20),
            };
        }
        SettingField::Difficulty => {
            settings.difficulty_enabled = !settings.difficulty_enabled;
        }
        SettingField::Theme => {
            settings.theme_default = match direction {
                AdjustmentDirection::Increase => settings.theme_default.next(),
                AdjustmentDirection::Decrease => settings.theme_default.previous(),
            };
        }
        SettingField::AudioEnabled => {
            settings.audio_enabled = !settings.audio_enabled;
        }
        SettingField::MusicVolume => {
            let step = 5;
            settings.music_volume = match direction {
                AdjustmentDirection::Increase => {
                    settings.music_volume.saturating_add(step).min(100)
                }
                AdjustmentDirection::Decrease => settings.music_volume.saturating_sub(step),
            };
        }
        SettingField::SfxVolume => {
            let step = 5;
            settings.sfx_volume = match direction {
                AdjustmentDirection::Increase => settings.sfx_volume.saturating_add(step).min(100),
                AdjustmentDirection::Decrease => settings.sfx_volume.saturating_sub(step),
            };
        }
    }
    *settings != original
}

pub fn format_field_value(settings: &Settings, field: SettingField) -> String {
    match field {
        SettingField::Mode => settings.mode.label().to_string(),
        SettingField::GridPreset => settings.grid_preset.label().to_string(),
        SettingField::BaseSpeed => format!("{:.3}s", settings.base_speed),
        SettingField::Difficulty => {
            if settings.difficulty_enabled {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }
        SettingField::Theme => settings.theme_default.label().to_string(),
        SettingField::AudioEnabled => {
            if settings.audio_enabled {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }
        SettingField::MusicVolume => format!("{}%", settings.music_volume),
        SettingField::SfxVolume => format!("{}%", settings.sfx_volume),
    }
}
