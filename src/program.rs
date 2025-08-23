use crate::ringbuffer::RingBuffer;

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
            'x' => {
                let val1 = self.pop();
                let val2 = self.pop();
                self.push(val1);
                self.push(val2);
            }
            '^' => {
                let (a, b) = self.pop2();
                self.push(((a as i32) ^ (b as i32)) as f32);
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
            _ => {}
        }
    }

    pub fn get_stack(&self) -> Vec<f32> {
        if self.stack.len() > 2 {
            self.stack.clone()
        } else {
            let mut ring_buffer_iter = self.ring_buffer.clone().into_iter();
            if self.stack.len() == 2 {
                vec![
                    self.stack[0],
                    self.stack[1],
                    ring_buffer_iter.next().unwrap(),
                ]
            } else if self.stack.len() == 1 {
                vec![
                    self.stack[0],
                    ring_buffer_iter.next().unwrap(),
                    ring_buffer_iter.next().unwrap(),
                ]
            } else {
                vec![
                    ring_buffer_iter.next().unwrap(),
                    ring_buffer_iter.next().unwrap(),
                    ring_buffer_iter.next().unwrap(),
                ]
            }
        }
    }
}

pub fn execute_string(input: &str, x: i32, y: i32, t: f64) -> Stack {
    let mut stack = Stack::new([x as f32, y as f32, t as f32]);
    stack.push(t as f32);
    stack.push(y as f32);
    stack.push(x as f32);

    for c in input.chars().rev() {
        stack.execute(c);
    }

    stack
}
