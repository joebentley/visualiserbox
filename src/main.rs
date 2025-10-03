mod app;
mod config;
mod drawing;
mod program;
mod recorder;
mod rect;
mod ringbuffer;
mod sound;
mod texteditor;

use crate::drawing::draw_text;
use std::sync::mpsc;

use raylib::prelude::*;

const DEJAVU_SANS: &[u8] = include_bytes!("DejaVuSans.ttf");

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
    if config.sound {
        stream.play();
    }
    let mut data = [0i16; MAX_SAMPLES_PER_UPDATE as usize];

    let mut app_state =
        crate::app::AppState::new(screen_recorder_length, progress_sender, progress_receiver);

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

        if config.sound && stream.is_processed() {
            frames = sound::fill_buffer(&mut data, app_state.current_input_line(), frames, mx, my);
            stream.update(&data[..MAX_SAMPLES_PER_UPDATE as usize / 2]);
        }

        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::WHITE);

            let mut debug_string = String::new();

            for y in 0..scaled_height {
                for x in 0..scaled_width {
                    let mut stack = program::execute_string(
                        app_state.current_input_line(),
                        [x as f32, y as f32, t as f32],
                    );

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
            app_state.draw_input_text(&mut d, &font, 20, 20, 40);
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
