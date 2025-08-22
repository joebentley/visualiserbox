pub struct Stack {
    stack: Vec<f32>,
    ring_buffer: [f32; 3],
    ring_buffer_pointer: usize,
}

impl Stack {
    pub fn new(ring_buffer: [f32; 3]) -> Stack {
        Stack {
            stack: Vec::new(),
            ring_buffer,
            ring_buffer_pointer: 0,
        }
    }

    pub fn push(&mut self, val: f32) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> f32 {
        match self.stack.pop() {
            Some(v) => v,
            None => self.end_repeat(),
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
            _ => {}
        }
    }

    pub fn get_stack(&self) -> Vec<f32> {
        if self.stack.len() > 2 {
            self.stack.clone()
        } else if self.stack.len() == 2 {
            vec![
                self.stack[0],
                self.stack[1],
                self.ring_buffer[self.ring_buffer_pointer],
            ]
        } else if self.stack.len() == 1 {
            vec![
                self.stack[0],
                self.ring_buffer[self.ring_buffer_pointer],
                self.ring_buffer[(self.ring_buffer_pointer + 1) % 3],
            ]
        } else {
            vec![
                self.ring_buffer[self.ring_buffer_pointer],
                self.ring_buffer[(self.ring_buffer_pointer + 1) % 3],
                self.ring_buffer[(self.ring_buffer_pointer + 2) % 3],
            ]
        }
    }

    fn end_repeat(&mut self) -> f32 {
        let val = self.ring_buffer[self.ring_buffer_pointer];
        self.ring_buffer_pointer = (self.ring_buffer_pointer + 1) % 3;
        val
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
