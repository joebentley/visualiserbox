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

        let mouse_position = d.get_mouse_position();
        let (mx, my) = (
            mouse_position.x.floor() as i32 / scale,
            mouse_position.y.floor() as i32 / scale,
        );

        let mut debug_string = String::new();

        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let mut stack = program::execute_string(input.as_str(), x, y, t);

                if y == my && x == mx {
                    for value in &stack.stack {
                        debug_string += (value.to_string() + "\n").as_str();
                    }
                    debug_string += format!("{} {}", mx, my).as_str();
                }

                let (h, s, v) = (stack.pop_or(0.0), stack.pop_or(0.0), stack.pop_or(1.0));

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

        if let Some(c) = d.get_char_pressed() {
            input = c.to_string() + &input;
        } else if !input.is_empty()
            && (d.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || d.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
        {
            input = input[1..].to_string();
        }

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
