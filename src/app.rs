use crate::program;
use crate::recorder;
use raylib::{ffi::KeyboardKey, RaylibHandle};

pub trait InputProvider {
    fn is_key_down(&self, key: KeyboardKey) -> bool;
    fn is_key_pressed(&self, key: KeyboardKey) -> bool;
    fn is_key_pressed_repeat(&self, key: KeyboardKey) -> bool;
    fn get_key_pressed(&mut self) -> Option<char>;
    fn get_char_pressed(&mut self) -> Option<char>;

    fn keystring(&mut self) -> Option<String> {
        let mut s = String::new();

        if self.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || self.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        {
            s = "C-".to_string();
        } else if self.is_key_down(KeyboardKey::KEY_LEFT_ALT)
            || self.is_key_down(KeyboardKey::KEY_RIGHT_ALT)
        {
            s = "M-".to_string();
        }

        // Need to handle the next character specially here if either were
        // pressed since get_char_pressed will be None if ctrl or alt is
        // held down. Note, this assumes QWERTY layout
        if !s.is_empty()
            && let Some(mut c) = self.get_key_pressed()
        {
            c.make_ascii_lowercase();
            if c.is_alphanumeric() {
                return Some(s + &c.to_string());
            } else {
                return None;
            }
        }

        self.get_char_pressed().map(|c| c.to_string())
    }
}

impl InputProvider for RaylibHandle {
    fn is_key_down(&self, key: KeyboardKey) -> bool {
        self.is_key_down(key)
    }
    fn is_key_pressed(&self, key: KeyboardKey) -> bool {
        self.is_key_pressed(key)
    }
    fn is_key_pressed_repeat(&self, key: KeyboardKey) -> bool {
        self.is_key_pressed_repeat(key)
    }
    fn get_key_pressed(&mut self) -> Option<char> {
        self.get_key_pressed_number().map(|c| c as u8 as char)
    }
    fn get_char_pressed(&mut self) -> Option<char> {
        self.get_char_pressed()
    }
}

pub trait TimeProvider {
    fn get_time(&self) -> f64;
}

impl TimeProvider for RaylibHandle {
    fn get_time(&self) -> f64 {
        self.get_time()
    }
}

pub struct AppState {
    pub input: String,
    pub screen_recorder: recorder::ScreenRecorder,
    pub screen_recorder_state: recorder::ScreenRecorderState,
    pub time_offset: f64,
}

fn next_available_video_path() -> std::io::Result<Option<std::path::PathBuf>> {
    let cwd = std::env::current_dir()?;
    for i in 0..999 {
        let video_name = format!("video_{:03}.mp4", i);
        let mut video_path = cwd.clone();
        video_path.push(video_name);
        if !video_path.try_exists()? {
            return Ok(Some(video_path));
        }
    }
    Ok(None)
}

impl AppState {
    pub fn update<T: InputProvider + TimeProvider>(
        &mut self,
        provider: &mut T,
    ) -> anyhow::Result<()> {
        if self.screen_recorder_state.is_saving() {
            self.screen_recorder_state.update();
        }

        if let Some(s) = provider.keystring() {
            match s.as_str() {
                "C-s" => {
                    if !self.screen_recorder_state.is_saving()
                        && let Some(path) = next_available_video_path()?
                    {
                        self.screen_recorder_state.start();
                        self.screen_recorder.save_as_video(path.to_str().unwrap());
                    }
                }
                "C-k" => {
                    self.input.clear();
                }
                "C-t" => {
                    self.time_offset = provider.get_time();
                }
                &_ => {
                    if program::ALLOWED.contains(&s.chars().nth(0).unwrap_or('§')) {
                        self.input = s + &self.input
                    }
                }
            }
        }

        if !self.input.is_empty()
            && (provider.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || provider.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
        {
            self.input = self.input[1..].to_string();
        }
        Ok(())
    }
}
