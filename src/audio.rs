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
        let eat = load_optional_sound(&format!("{asset_dir}/eat.wav")).await;
        let death = load_optional_sound(&format!("{asset_dir}/death.wav")).await;
        let pause = load_optional_sound(&format!("{asset_dir}/pause.wav")).await;
        let resume = load_optional_sound(&format!("{asset_dir}/resume.wav")).await;
        let bgm = load_optional_sound(&format!("{asset_dir}/bgm.ogg")).await;
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

async fn load_optional_sound(path: &str) -> Option<Sound> {
    match load_sound(path).await {
        Ok(sound) => Some(sound),
        Err(err) => {
            eprintln!("warning: failed to load sound at {path}: {err}");
            None
        }
    }
}
