use raylib::color::Color;

use crate::ringbuffer::RingBuffer;

pub const ALLOWED: [char; 20] = [
    'x', 'd', '.', 't', 'q', '^', '&', '|', '+', '-', '*', '/', 'l', 'e', 'c', 'm', '%', 'r', 'n',
    'b',
];

pub struct Stack {
    stack: Vec<f32>,
    ring_buffer: RingBuffer<f32>,
}

impl Stack {
    pub fn new(ring_buffer: [f32; 3]) -> Stack {
        Stack {
            stack: Vec::new(),
            ring_buffer: ring_buffer.into_iter().collect(),
        }
    }

    pub fn push(&mut self, val: f32) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> f32 {
        match self.stack.pop() {
            Some(v) => v,
            None => {
                let val = *self.ring_buffer.current();
                self.ring_buffer.increment();
                val
            }
        }
    }

    pub fn pop2(&mut self) -> (f32, f32) {
        (self.pop(), self.pop())
    }

    pub fn execute(&mut self, instruction: char) {
        match instruction {
            // Stack functions
            'x' => {
                // swap
                let val1 = self.pop();
                let val2 = self.pop();
                self.push(val1);
                self.push(val2);
            }
            'd' => {
                // duplicate
                let val = self.pop();
                self.push(val);
                self.push(val);
            }
            '.' => {
                // pop
                self.pop();
            }
            't' => {
                // tuck x1 x2 -- x1 x2 x1
                let val1 = self.pop();
                let val2 = self.pop();
                self.push(val1);
                self.push(val2);
                self.push(val1);
            }
            'q' => {
                // duplicate under x1 x2 -- x1 x2 x2
                let val1 = self.pop();
                let val2 = self.pop();
                self.push(val2);
                self.push(val2);
                self.push(val1);
            }
            // Maths functions
            '^' => {
                let (a, b) = self.pop2();
                self.push(((a as i32) ^ (b as i32)) as f32);
            }
            '&' => {
                let (a, b) = self.pop2();
                self.push(((a as i32) & (b as i32)) as f32);
            }
            '|' => {
                let (a, b) = self.pop2();
                self.push(((a as i32) | (b as i32)) as f32);
            }
            '+' => {
                let val = self.pop() + self.pop();
                self.push(val);
            }
            '-' => {
                let val = self.pop() - self.pop();
                self.push(val);
            }
            '*' => {
                let val = self.pop() * self.pop();
                self.push(val);
            }
            '/' => {
                let val = self.pop();
                let mut val2 = self.pop();
                if val2 == 0.0 {
                    val2 = 1.0;
                }
                self.push(val / val2);
            }
            'l' => {
                let val = self.pop();
                let mut val2 = self.pop();
                if val2 < 1.0 {
                    val2 = 1.0;
                }
                self.push(val * val2.ln());
            }
            'e' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(val * val2.exp());
            }
            'c' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(val * val2.cos());
            }
            'm' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(val.max(val2));
            }
            '%' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(val % val2);
            }
            'r' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(val2 * rand::random::<f32>() + val);
            }
            'n' => {
                let val = self.pop();
                self.push(-val);
            }
            // Lock brightness
            'b' => {
                let val = self.pop();
                let val2 = self.pop();
                self.push(1.0);
                self.push(val2);
                self.push(val);
            }
            _ => {}
        }
    }
}

pub fn execute_string(input: &str, initial_values: [f32; 3]) -> Stack {
    let mut stack = Stack::new(initial_values);
    stack.push(initial_values[2]);
    stack.push(initial_values[1]);
    stack.push(initial_values[0]);

    for c in input.chars().rev() {
        stack.execute(c);
    }

    stack
}

// Adapted from https://github.com/raysan5/raylib/blob/16a0b966c3640d679a9bce5c11164945cadd0783/src/rtextures.c#L4959
fn color_from_hsv(h: f32, s: f32, v: f32) -> Color {
    let mut color = Color::new(0, 0, 0, 255);

    // Red channel
    let mut k = (5.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.r = (((v - v * s * k) * 255.0) % 256.0) as u8;

    // Green channel
    let mut k = (3.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.g = (((v - v * s * k) * 255.0) % 256.0) as u8;

    // Blue channel
    let mut k = (1.0 + h / 60.0) % 6.0;
    let t = 4.0 - k;
    k = k.min(t);
    k = k.min(1.0);
    k = k.max(0.0);
    color.b = (((v - v * s * k) * 255.0) % 256.0) as u8;

    color
}

pub fn execute_string_to_color(input: &str, initial_values: [f32; 3]) -> Color {
    let mut stack = execute_string(input, initial_values);
    color_from_hsv(stack.pop(), stack.pop(), stack.pop())
}
