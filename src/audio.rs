use std::path::{Path, PathBuf};

use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume};

pub struct AudioManager {
    eat: Option<Sound>,
    death: Option<Sound>,
    pause: Option<Sound>,
    resume: Option<Sound>,
    bgm: Option<Sound>,
    bgm_started: bool,
}

impl AudioManager {
    pub async fn load(asset_dir: &str) -> Self {
        let candidate_dirs = candidate_asset_dirs(asset_dir);

        let eat = load_optional_sound(&candidate_dirs, &["eat.wav"]).await;
        let death = load_optional_sound(&candidate_dirs, &["death.wav"]).await;
        let pause = load_optional_sound(&candidate_dirs, &["pause.wav"]).await;
        let resume = load_optional_sound(&candidate_dirs, &["resume.wav"]).await;
        let bgm = load_optional_sound(&candidate_dirs, &["bgm.ogg", "bgm.wav"]).await;
        Self {
            eat,
            death,
            pause,
            resume,
            bgm,
            bgm_started: false,
        }
    }

    pub fn play_eat(&self, enabled: bool, sfx_volume: u8) {
        self.play_sfx(self.eat.as_ref(), enabled, sfx_volume);
    }

    pub fn play_death(&self, enabled: bool, sfx_volume: u8) {
        self.play_sfx(self.death.as_ref(), enabled, sfx_volume);
    }

    pub fn play_pause(&self, enabled: bool, sfx_volume: u8) {
        self.play_sfx(self.pause.as_ref(), enabled, sfx_volume);
    }

    pub fn play_resume(&self, enabled: bool, sfx_volume: u8) {
        self.play_sfx(self.resume.as_ref(), enabled, sfx_volume);
    }

    pub fn update_bgm(&mut self, enabled: bool, music_volume: u8, ducked: bool) {
        let Some(sound) = self.bgm.as_ref() else {
            return;
        };

        if !self.bgm_started {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 0.0,
                },
            );
            self.bgm_started = true;
        }

        let mut volume = if enabled {
            f32::from(music_volume) / 100.0
        } else {
            0.0
        };
        if ducked {
            volume *= 0.7;
        }
        set_sound_volume(sound, volume);
    }

    fn play_sfx(&self, sound: Option<&Sound>, enabled: bool, sfx_volume: u8) {
        if !enabled {
            return;
        }
        let Some(sound) = sound else {
            return;
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: f32::from(sfx_volume) / 100.0,
            },
        );
    }
}

fn candidate_asset_dirs(asset_dir: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(PathBuf::from(asset_dir));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push(exe_dir.join(asset_dir));
            dirs.push(exe_dir.join("..").join(asset_dir));
            dirs.push(exe_dir.join("..").join("..").join(asset_dir));
        }
    }

    dirs
}

async fn load_optional_sound(candidate_dirs: &[PathBuf], names: &[&str]) -> Option<Sound> {
    for dir in candidate_dirs {
        for name in names {
            let path = dir.join(name);
            if !path.is_file() {
                continue;
            }

            let path_str = path_to_string(&path);
            match load_sound(&path_str).await {
                Ok(sound) => return Some(sound),
                Err(err) => {
                    eprintln!("warning: failed to load sound at {}: {err}", path.display());
                }
            }
        }
    }

    eprintln!("warning: missing sound asset(s): {}", names.join(" or "));
    None
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
