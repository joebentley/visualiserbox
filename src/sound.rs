use raylib::{
    color::Color,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

use crate::rect::IRect;

fn map(input_start: f32, input_end: f32, output_start: f32, output_end: f32, input: f32) -> f32 {
    output_start + ((output_end - output_start) * (input - input_start)) / (input_end - input_start)
}

pub fn draw_buffer(d: &mut RaylibDrawHandle, audio_buffer: &[i16], bounds: IRect) {
    d.draw_rectangle(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        Color::MEDIUMSPRINGGREEN.alpha(0.3),
    );

    if audio_buffer.is_empty() {
        return;
    }

    let biggest_sample = *audio_buffer.iter().max().unwrap();

    for x in 0..bounds.w {
        let sample_index = map(
            0.0,
            bounds.w as f32,
            0.0,
            audio_buffer.len() as f32,
            x as f32,
        )
        .floor() as usize;

        let sample = audio_buffer[sample_index];
        let y = map(
            -biggest_sample as f32,
            biggest_sample as f32,
            bounds.h as f32 - 4.0,
            2.0,
            sample as f32,
        )
        .round() as i32;

        d.draw_rectangle(x + bounds.x, y + bounds.y, 2, 2, Color::GREENYELLOW);
    }
}
