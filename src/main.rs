mod config;
mod program;
//mod recorder;
mod ringbuffer;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
//use std::sync::mpsc;
use std::time::Duration;

use sdl2::{pixels::Color, rect::Rect, ttf::Font};

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

// Adapted from https://github.com/raysan5/raylib/blob/16a0b966c3640d679a9bce5c11164945cadd0783/src/rtextures.c#L4959
fn color_from_hsv(h: f32, s: f32, v: f32) -> Color {
    let mut color = Color::RGBA(0, 0, 0, 255);

    // Red channel
    let mut k = (5.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.r = ((v - v * s * k) * 255.0) as u8;

    // Green channel
    let mut k = (3.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.g = ((v - v * s * k) * 255.0) as u8;

    // Blue channel
    let mut k = (1.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.b = ((v - v * s * k) * 255.0) as u8;

    color
}

const DEJAVU_SANS: &[u8] = include_bytes!("DejaVuSans.ttf");

fn draw_text(
    canvas: &mut WindowCanvas,
    font: &Font,
    text: impl AsRef<str>,
    x: i32,
    y: i32,
) -> anyhow::Result<()> {
    if text.as_ref().is_empty() {
        return Ok(());
    }
    let texture_creator = canvas.texture_creator();
    let texture = font
        .render(text.as_ref())
        .solid(Color::WHITE)?
        .as_texture(&texture_creator)?;
    let query = texture.query();
    let w = query.width;
    let h = query.height;
    canvas
        .copy(&texture, None, Rect::new(x, y, w, h))
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

struct AppState {
    input: String,
    //screen_recorder: recorder::ScreenRecorder,
    //screen_recorder_state: recorder::ScreenRecorderState,
    time_offset: std::time::Instant,
}

impl AppState {
    fn update(
        &mut self,
        keystring: impl AsRef<str>,
        time: std::time::Instant,
    ) -> anyhow::Result<()> {
        // if self.screen_recorder_state.is_saving() {
        //     self.screen_recorder_state.update();
        // }

        let s = keystring.as_ref();

        match s {
            // "C-s" => {
            //     if !self.screen_recorder_state.is_saving()
            //         && let Some(path) = next_available_video_path()?
            //     {
            //         self.screen_recorder_state.start();
            //         self.screen_recorder.save_as_video(path.to_str().unwrap());
            //     }
            //}
            "C-k" => {
                self.input.clear();
            }
            "C-t" => {
                self.time_offset = time;
            }
            "DEL" => {
                if !self.input.is_empty() {
                    self.input = self.input[1..].to_string();
                }
            }
            &_ => {
                if program::ALLOWED.contains(&s.chars().nth(0).unwrap_or('§')) {
                    self.input = s.to_string() + &self.input;
                }
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    colog::init();

    ffmpeg_sidecar::download::auto_download()?;

    let config = config::Config::from_file("config.toml")?;

    let sdl_context = sdl2::init().map_err(anyhow::Error::msg)?;
    let sdl_ttf_context = sdl2::ttf::init().map_err(anyhow::Error::msg)?;
    let video_subsystem = sdl_context.video().unwrap();

    let width = 640;
    let height = 480;
    let window = video_subsystem
        .window("visualiserbox", width, height)
        .position_centered()
        .build()
        .unwrap();

    let rwops = sdl2::rwops::RWops::from_bytes(DEJAVU_SANS).map_err(anyhow::Error::msg)?;
    let font = sdl_ttf_context
        .load_font_from_rwops(rwops, 32)
        .map_err(anyhow::Error::msg)?;

    let mut canvas = window.into_canvas().build().map_err(anyhow::Error::msg)?;

    let scale = config.scale;
    let scaled_width = width / scale;
    let scaled_height = height / scale;

    let screen_recorder_length = config.video_frames as usize;

    //let (progress_sender, progress_receiver) = mpsc::channel();

    let mut app_state = AppState {
        input: String::new(),
        //screen_recorder: recorder::ScreenRecorder::new(screen_recorder_length, progress_sender),
        //screen_recorder_state: recorder::ScreenRecorderState::new(progress_receiver),
        time_offset: std::time::Instant::now(),
    };

    let mut event_pump = sdl_context.event_pump().map_err(anyhow::Error::msg)?;

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    'running: loop {
        let mut read_char = String::new();
        let mut keystring = String::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::TextInput { text, .. } => {
                    read_char = text;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if k == Keycode::LCTRL || k == Keycode::RCTRL {
                        keystring = "C-".to_string();
                    } else if k == Keycode::LALT || k == Keycode::RALT {
                        keystring = "M-".to_string();
                    } else if k == Keycode::BACKSPACE {
                        keystring = "DEL".to_string();
                    }
                }
                _ => {}
            }
        }

        if keystring != "DEL" {
            keystring += &read_char;
        }

        let mouse_state = event_pump.mouse_state();
        let (mx, my) = (mouse_state.x(), mouse_state.y());

        let t = app_state.time_offset.elapsed().as_secs_f64();

        app_state.update(keystring, std::time::Instant::now())?;

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        //canvas.present();

        let mut debug_string = String::new();

        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let mut stack = program::execute_string(&app_state.input, x, y, t);

                if y == my as u32 && x == mx as u32 {
                    for value in stack.get_stack() {
                        debug_string += (value.to_string() + "\n").as_str();
                    }
                    debug_string += format!("{} {}", mx, my).as_str();
                }

                let (h, s, v) = (stack.pop(), stack.pop(), stack.pop());

                canvas.set_draw_color(color_from_hsv(h, s, v));
                canvas
                    .fill_rect(Rect::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                        scale,
                        scale,
                    ))
                    .map_err(anyhow::Error::msg)?;
            }
        }

        draw_text(&mut canvas, &font, debug_string, width as i32 - 150, 20)?;
        draw_text(&mut canvas, &font, &app_state.input, 20, 20)?;
        // if config.show_fps {
        //     draw_text(
        //         &mut canvas,
        //         &font,
        //         fps.round().to_string(),
        //         width as i32 - 80,
        //         400,
        //     );
        // }

        // if app_state.screen_recorder_state.is_saving() {
        //     let text = app_state
        //         .screen_recorder_state
        //         .progress_string(screen_recorder_length);
        //     draw_text(&mut d, &font, text, 10, 400, 30);
        // }

        // app_state
        //     .screen_recorder
        //     .push_image(rl.load_image_from_screen(&thread).clone());

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
