use raylib::prelude::*;

pub fn draw_text(
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
