mod config;
mod program;
mod recorder;
mod ringbuffer;

use std::sync::mpsc;

use raylib::prelude::*;

fn next_available_video_path() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().unwrap();
    for i in 0..999 {
        let video_name = format!("video_{:03}.mp4", i);
        let mut video_path = cwd.clone();
        video_path.push(video_name);
        if !video_path.try_exists().unwrap() {
            return Some(video_path);
        }
    }
    None
}

fn path_relative_to_executable(font_file_name: &str) -> std::path::PathBuf {
    let mut cwd = std::env::current_exe().unwrap();
    cwd.pop();
    cwd.push(font_file_name);
    cwd
}

fn main() -> anyhow::Result<()> {
    colog::init();

    ffmpeg_sidecar::download::auto_download().unwrap();

    let config = config::Config::from_file("config.toml")?;

    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    let scale = config.scale as i32;
    let scaled_width = width / scale;
    let scaled_height = height / scale;

    let mut input = String::new();

    let font = rl
        .load_font(
            &thread,
            path_relative_to_executable("DejaVuSans.ttf")
                .to_str()
                .unwrap(),
        )
        .unwrap();

    let screen_recorder_length = config.video_frames as usize;
    let mut screen_recorder = recorder::ScreenRecorder::new(screen_recorder_length);

    let (progress_sender, progress_receiver) = mpsc::channel();
    let mut screen_recorder_state = recorder::ScreenRecorderState::new(progress_receiver);

    while !rl.window_should_close() {
        let fps = 1.0 / rl.get_frame_time();
        let t = rl.get_time();

        let mouse_position = rl.get_mouse_position();
        let (mx, my) = (
            mouse_position.x.floor() as i32 / scale,
            mouse_position.y.floor() as i32 / scale,
        );

        if screen_recorder_state.is_saving() {
            screen_recorder_state.update();
        }

        if let Some(c) = rl.get_char_pressed() {
            if c == 's' {
                if !screen_recorder_state.is_saving()
                    && let Some(path) = next_available_video_path()
                {
                    screen_recorder_state.start();
                    screen_recorder.save_as_video(path.to_str().unwrap(), progress_sender.clone());
                }
            } else {
                input = c.to_string() + &input;
            }
        } else if !input.is_empty()
            && (rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
        {
            input = input[1..].to_string();
        }

        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::WHITE);

            let mut debug_string = String::new();

            for y in 0..scaled_height {
                for x in 0..scaled_width {
                    let mut stack = program::execute_string(input.as_str(), x, y, t);

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
            d.draw_text_ex(
                &font,
                debug_string.as_str(),
                Vector2::new(width as f32 - 150.0, 20.0),
                30.0,
                0.0,
                Color::NAVAJOWHITE,
            );

            d.draw_text_ex(
                &font,
                input.as_str(),
                Vector2::new(20.0, 20.0),
                40.0,
                0.0,
                Color::NAVAJOWHITE,
            );

            if config.show_fps {
                d.draw_text_ex(
                    &font,
                    fps.round().to_string().as_str(),
                    Vector2::new(width as f32 - 80.0, 400.0),
                    40.0,
                    0.0,
                    Color::NAVAJOWHITE,
                );
            }

            if screen_recorder_state.is_saving() {
                let text = screen_recorder_state.progress_string(screen_recorder_length);
                d.draw_text_ex(
                    &font,
                    text.as_str(),
                    Vector2::new(10.0, 400.0),
                    30.0,
                    0.0,
                    Color::NAVAJOWHITE,
                );
            }
        }

        screen_recorder.push_image(rl.load_image_from_screen(&thread).clone());
    }

    Ok(())
}
