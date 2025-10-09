use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::program;
use crate::recorder;
use crate::recorder::ScreenRecorder;
use crate::recorder::ScreenRecorderMessage;
use crate::recorder::ScreenRecorderState;
use crate::texteditor::TextEditor;
use crate::utils;
use raylib::prelude::*;

pub trait InputProvider {
    fn is_key_down(&self, key: KeyboardKey) -> bool;
    fn is_key_pressed(&self, key: KeyboardKey) -> bool;
    fn is_key_pressed_repeat(&self, key: KeyboardKey) -> bool;
    fn get_key_pressed(&mut self) -> Option<KeyboardKey>;
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
            && let Some(c) = self.get_key_pressed()
        {
            match c {
                KeyboardKey::KEY_UP => return Some(s + "<up>"),
                KeyboardKey::KEY_DOWN => return Some(s + "<down>"),
                _ => {}
            }

            let mut c = c as u32 as u8 as char;

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
    fn get_key_pressed(&mut self) -> Option<KeyboardKey> {
        self.get_key_pressed()
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

struct ProgramAnimator {
    t: f32,
    sequence_speed: f32,
}

impl ProgramAnimator {
    pub fn new(sequence_speed: f32) -> Self {
        Self {
            t: 0.0,
            sequence_speed,
        }
    }

    pub fn get_blend_mode(
        &self,
        current_program: impl Into<String>,
        next_program: impl Into<String>,
    ) -> program::BlendMode {
        if self.t < 0.7 {
            program::BlendMode::One(current_program.into())
        } else {
            program::BlendMode::Two(
                current_program.into(),
                next_program.into(),
                utils::map(0.7, 1.0, 0.0, 1.0, self.t),
            )
        }
    }

    pub fn tick(&mut self) {
        self.t += self.sequence_speed;
    }

    pub fn needs_program(&self) -> bool {
        self.t >= 1.0
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
    }
}

pub struct AppState {
    text_editor: TextEditor,
    program_animator: ProgramAnimator,
    pub screen_recorder: recorder::ScreenRecorder,
    pub screen_recorder_state: recorder::ScreenRecorderState,
    pub time_offset: f64,
    pub time_multiplier: f64,
}

impl AppState {
    pub fn new(
        screen_recorder_length: usize,
        progress_sender: Sender<ScreenRecorderMessage>,
        progress_receiver: Receiver<ScreenRecorderMessage>,
        sequence_speed: f32,
    ) -> Self {
        Self {
            text_editor: TextEditor::new(),
            program_animator: ProgramAnimator::new(sequence_speed),
            screen_recorder: ScreenRecorder::new(screen_recorder_length, progress_sender),
            screen_recorder_state: ScreenRecorderState::new(progress_receiver),
            time_offset: 0.0,
            time_multiplier: 1.0,
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
                    if let Some(file) = rfd::FileDialog::new().save_file() {
                        let mut file = File::create(file)?;
                        write!(file, "{}", self.text_editor)?;
                    }
                }
                "C-o" => {
                    if let Some(file) = rfd::FileDialog::new().pick_file() {
                        let mut file = File::open(file)?;
                        let mut s = String::new();
                        file.read_to_string(&mut s)?;
                        self.text_editor.load_from_string(s);
                    }
                }
                "M-s" => {
                    if !self.screen_recorder_state.is_saving()
                        && let Some(file) = rfd::FileDialog::new()
                            .add_filter("mp4", &["mp4"])
                            .set_file_name("output.mp4")
                            .save_file()
                    {
                        self.screen_recorder_state.start();
                        self.screen_recorder.save_as_video(file.to_str().unwrap());
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
                "M-<up>" => {
                    self.time_multiplier += 0.1;
                }
                "M-<down>" => {
                    self.time_multiplier -= 0.1;
                }
                &_ => {
                    if program::ALLOWED.contains(&s.chars().nth(0).unwrap_or('§')) {
                        self.text_editor.insert_char(s.chars().nth(0).unwrap());
                    }
                }
            }
        } else if provider.is_key_pressed(KeyboardKey::KEY_DOWN) {
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

        if self.text_editor.num_non_empty_lines() >= 2 {
            self.program_animator.tick();
            if self.program_animator.needs_program() {
                self.text_editor.goto_next_nonempty();
                self.program_animator.reset();
            }
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

    pub fn get_blend_mode(&self) -> program::BlendMode {
        let current = self.text_editor.current_line();
        if self.text_editor.num_non_empty_lines() <= 1 {
            return program::BlendMode::One(current.to_owned());
        }

        let next = self.text_editor.get_next_nonempty();
        self.program_animator.get_blend_mode(current, next)
    }
}
