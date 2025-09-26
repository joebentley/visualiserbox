mod config;
mod program;
mod recorder;
mod rect;
mod ringbuffer;
mod sound;

use std::sync::mpsc;

use raylib::prelude::*;

use crate::sound::process_sample;

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

const DEJAVU_SANS: &[u8] = include_bytes!("DejaVuSans.ttf");

fn draw_text(
    d: &mut RaylibDrawHandle,
    font: &Font,
    text: impl AsRef<str>,
    x: i32,
    y: i32,
    size: i32,
) {
    d.draw_text_ex(
        font,
        text.as_ref(),
        Vector2::new(x as f32, y as f32),
        size as f32,
        0.0,
        Color::NAVAJOWHITE,
    );
}

fn keystring(rl: &mut RaylibHandle) -> Option<String> {
    let mut s = String::new();

    if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
    {
        s = "C-".to_string();
    } else if rl.is_key_down(KeyboardKey::KEY_LEFT_ALT)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_ALT)
    {
        s = "M-".to_string();
    }

    // Need to handle the next character specially here if either were
    // pressed since get_char_pressed will be None if ctrl or alt is
    // held down. Note, this assumes QWERTY layout
    if !s.is_empty()
        && let Some(c) = rl.get_key_pressed_number()
    {
        let mut c = c as u8 as char;
        c.make_ascii_lowercase();
        if c.is_alphanumeric() {
            return Some(s + &c.to_string());
        } else {
            return None;
        }
    }

    rl.get_char_pressed().map(|c| c.to_string())
}

struct AppState {
    input: String,
    screen_recorder: recorder::ScreenRecorder,
    screen_recorder_state: recorder::ScreenRecorderState,
    time_offset: f64,
}

impl AppState {
    fn update(&mut self, rl: &mut RaylibHandle) -> anyhow::Result<()> {
        if self.screen_recorder_state.is_saving() {
            self.screen_recorder_state.update();
        }

        if let Some(s) = keystring(rl) {
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
                    self.time_offset = rl.get_time();
                }
                &_ => {
                    if program::ALLOWED.contains(&s.chars().nth(0).unwrap_or('§')) {
                        self.input = s + &self.input
                    }
                }
            }
        }

        if !self.input.is_empty()
            && (rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
        {
            self.input = self.input[1..].to_string();
        }
        Ok(())
    }
}

const MAX_SAMPLES_PER_UPDATE: u32 = 2048;

fn main() -> anyhow::Result<()> {
    colog::init();

    ffmpeg_sidecar::download::auto_download()?;

    let config = config::Config::from_file("config.toml")?;

    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    let scale = config.scale as i32;
    let scaled_width = width / scale;
    let scaled_height = height / scale;

    let font = rl.load_font_from_memory(&thread, ".ttf", DEJAVU_SANS, 32, None)?;

    let screen_recorder_length = config.video_frames as usize;

    let (progress_sender, progress_receiver) = mpsc::channel();

    let ra = raylib::core::audio::RaylibAudio::init_audio_device()?;
    ra.set_audio_stream_buffer_size_default(MAX_SAMPLES_PER_UPDATE as i32);
    let mut stream = ra.new_audio_stream(44100, 16, 1);
    stream.play();
    let mut data = [0i16; MAX_SAMPLES_PER_UPDATE as usize];

    let mut app_state = AppState {
        input: String::new(),
        screen_recorder: recorder::ScreenRecorder::new(screen_recorder_length, progress_sender),
        screen_recorder_state: recorder::ScreenRecorderState::new(progress_receiver),
        time_offset: 0.0,
    };

    let mut frames: u64 = 0;

    while !rl.window_should_close() {
        let fps = 1.0 / rl.get_frame_time();
        let t = rl.get_time() - app_state.time_offset;

        let mouse_position = rl.get_mouse_position();
        let (mx, my) = (
            mouse_position.x.floor() as i32 / scale,
            mouse_position.y.floor() as i32 / scale,
        );

        app_state.update(&mut rl)?;

        if stream.is_processed() {
            for frame in &mut data {
                *frame = process_sample(&app_state.input, frames, mx, my);
                frames += 1;
            }
            stream.update(&data[..MAX_SAMPLES_PER_UPDATE as usize / 2]);
        }

        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::WHITE);

            let mut debug_string = String::new();

            for y in 0..scaled_height {
                for x in 0..scaled_width {
                    let mut stack =
                        program::execute_string(&app_state.input, [x as f32, y as f32, t as f32]);

                    if y == my && x == mx {
                        for value in stack.get_stack() {
                            debug_string += (value.to_string() + "\n").as_str();
                        }
                        debug_string += format!("{} {}", mx, my).as_str();
                    }

                    let (h, s, v) = (stack.pop(), stack.pop(), stack.pop());

                    d.draw_rectangle(
                        x * scale,
                        y * scale,
                        scale,
                        scale,
                        Color::color_from_hsv(h, s, v),
                    );
                }
            }
            draw_text(&mut d, &font, debug_string, width - 150, 20, 30);
            draw_text(&mut d, &font, &app_state.input, 20, 20, 40);
            if config.show_fps {
                draw_text(&mut d, &font, fps.round().to_string(), width - 80, 400, 40);
            }

            if app_state.screen_recorder_state.is_saving() {
                let text = app_state
                    .screen_recorder_state
                    .progress_string(screen_recorder_length);
                draw_text(&mut d, &font, text, 10, 400, 30);
            }
        }

        app_state
            .screen_recorder
            .push_image(rl.load_image_from_screen(&thread).clone());
    }

    Ok(())
}
