use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::program;
use crate::recorder;
use crate::recorder::ScreenRecorder;
use crate::recorder::ScreenRecorderMessage;
use crate::recorder::ScreenRecorderState;
use crate::texteditor::TextEditor;
use raylib::prelude::*;

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
            if c.is_ascii() {
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
    text_editor: TextEditor,
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
    pub fn new(
        screen_recorder_length: usize,
        progress_sender: Sender<ScreenRecorderMessage>,
        progress_receiver: Receiver<ScreenRecorderMessage>,
    ) -> Self {
        Self {
            text_editor: TextEditor::new(),
            screen_recorder: ScreenRecorder::new(screen_recorder_length, progress_sender),
            screen_recorder_state: ScreenRecorderState::new(progress_receiver),
            time_offset: 0.0,
        }
    }

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
                "C-t" => {
                    self.time_offset = provider.get_time();
                }
                "C-r" => {
                    self.text_editor.randomise_line();
                }
                "C-[" => {
                    self.text_editor.rotate_line_left();
                }
                "C-]" => {
                    self.text_editor.rotate_line_right();
                }
                "C-p" => {
                    self.text_editor.prev_line();
                }
                "C-n" => {
                    self.text_editor.next_line();
                }
                "C-b" => {
                    self.text_editor.move_left();
                }
                "C-f" => {
                    self.text_editor.move_right();
                }
                "C-a" => {
                    self.text_editor.start_of_line();
                }
                "C-e" => {
                    self.text_editor.end_of_line();
                }
                "C-k" => {
                    self.text_editor.kill_to_end();
                }
                &_ => {
                    if program::ALLOWED.contains(&s.chars().nth(0).unwrap_or('§')) {
                        self.text_editor.insert_char(s.chars().nth(0).unwrap());
                    }
                }
            }
        }

        if provider.is_key_pressed(KeyboardKey::KEY_DOWN) {
            self.text_editor.next_line();
        } else if provider.is_key_pressed(KeyboardKey::KEY_UP) {
            self.text_editor.prev_line();
        } else if provider.is_key_pressed(KeyboardKey::KEY_LEFT) {
            self.text_editor.move_left();
        } else if provider.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            self.text_editor.move_right();
        } else if provider.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
            || provider.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
        {
            self.text_editor.backspace();
        }
        Ok(())
    }

    pub fn current_input_line(&self) -> &str {
        self.text_editor.current_line()
    }

    pub fn draw_input_text(
        &self,
        d: &mut RaylibDrawHandle,
        font: &Font,
        x: i32,
        y: i32,
        size: i32,
    ) {
        self.text_editor.draw(d, font, x, y, size);
    }
}
