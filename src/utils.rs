pub fn map(
    input_start: f32,
    input_end: f32,
    output_start: f32,
    output_end: f32,
    input: f32,
) -> f32 {
    output_start + ((output_end - output_start) * (input - input_start)) / (input_end - input_start)
}
