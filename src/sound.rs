use crate::program;

fn process_sample(input: impl AsRef<str>, frames: u64, x: i32, y: i32) -> i16 {
    let t: f32 = (frames as f32 / 44_100f32) % 2.0;
    let mut s = program::execute_string(input.as_ref(), [t, x as f32 / 100.0, y as f32 / 100.0]);
    let v1 = s.pop() % 1.0;
    let v2 = s.pop() % 0.7;
    let v3 = s.pop() % 0.5;
    ((v1 + v3 * v2) * 30_000f32).round() as i16
}

pub fn fill_buffer(data: &mut [i16], input: impl AsRef<str>, frames: u64, x: i32, y: i32) -> u64 {
    let mut frames = frames;
    for frame in data {
        *frame = process_sample(&input, frames, x, y);
        frames += 1;
    }
    frames
}
