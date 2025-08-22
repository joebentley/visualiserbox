pub struct Stack {
    pub stack: Vec<f32>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { stack: Vec::new() }
    }

    pub fn push(&mut self, val: f32) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> f32 {
        self.stack.pop().unwrap_or(0.0)
    }

    pub fn pop2(&mut self) -> (f32, f32) {
        (self.pop(), self.pop())
    }

    pub fn pop_or(&mut self, default: f32) -> f32 {
        self.stack.pop().unwrap_or(default)
    }

    pub fn execute(&mut self, instruction: char) {
        match instruction {
            '^' => {
                let (a, b) = self.pop2();
                self.push(((a as i32) ^ (b as i32)) as f32);
            }
            '+' => {
                let val = self.pop() + self.pop();
                self.push(val);
            }
            _ => {}
        }
    }
}

pub fn execute_string(input: &str, x: i32, y: i32, t: f64) -> Stack {
    let mut stack = Stack::new();
    stack.push(t as f32);
    stack.push(y as f32);
    stack.push(x as f32);

    for c in input.chars().rev() {
        stack.execute(c);
    }

    stack
}
