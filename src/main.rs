mod program;

use raylib::prelude::*;

fn path_relative_to_executable(font_file_name: &str) -> std::path::PathBuf {
    let mut cwd = std::env::current_exe().unwrap();
    cwd.pop();
    cwd.push(font_file_name);
    cwd
}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    let scale = 4;
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

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        let t = d.get_time();

        d.clear_background(Color::WHITE);

        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let (h, s, v) = program::execute_string(input.as_str(), x, y, t);

                // let x_val = (2.0 * (x as f64) * (4.0 + 2.0 * f64::sin(t))) as i32;
                // let y_val = (1.0 * (y as f64) * (6.0 + 2.0 * f64::sin(t + 0.2))) as i32;
                d.draw_rectangle(
                    x * scale,
                    y * scale,
                    scale,
                    scale,
                    Color::color_from_hsv(h, s, v),
                );
            }
        }

        if let Some(c) = d.get_char_pressed() {
            input = c.to_string() + &input;
        } else if !input.is_empty()
            && (d.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || d.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
        {
            input = input[1..].to_string();
        }

        let mouse_position = d.get_mouse_position();
        let (mx, my) = (mouse_position.x.floor(), mouse_position.y.floor());

        d.draw_text_ex(
            &font,
            format!("{} {}", mx, my).as_str(),
            Vector2::new(width as f32 - 150.0, height as f32 - 60.0),
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
    }
}
