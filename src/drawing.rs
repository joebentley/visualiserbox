use raylib::prelude::*;

pub fn draw_text(
    d: &mut RaylibDrawHandle,
    font: &Font,
    text: impl AsRef<str>,
    x: i32,
    y: i32,
    size: i32,
    colour: Color,
) {
    d.draw_text_ex(
        font,
        text.as_ref(),
        Vector2::new(x as f32, y as f32),
        size as f32,
        0.0,
        colour,
    );
}

pub fn draw_play_button(d: &mut RaylibDrawHandle, x: f32, y: f32, length: f32, colour: Color) {
    let v1 = Vector2::new(x, y);
    let v2 = Vector2::new(x, y + length);
    let v3 = Vector2::new(x + length, y + length / 2.0);
    d.draw_triangle(v1, v2, v3, colour.alpha(0.93));
}

pub fn draw_pause_button(d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32, colour: Color) {
    let space = w / 3;
    let width = space;
    d.draw_rectangle(x, y, width, h, colour.alpha(0.93));
    d.draw_rectangle(x + space + width, y, width, h, colour.alpha(0.93));
}
