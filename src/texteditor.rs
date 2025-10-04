use crate::{drawing::draw_text, program};
use rand::seq::IndexedRandom;
use raylib::prelude::*;

pub struct TextEditor {
    lines: Vec<String>,
    cursor: usize,
    current_line: usize,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new(); 10],
            cursor: 0,
            current_line: 0,
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, font: &Font, x: i32, y: i32, size: i32) {
        let line_height = font.measure_text("M", size as f32, 1.0).y as i32;

        self.draw_cursor(d, font, x, y, size);

        for (i, line) in self.lines.iter().enumerate() {
            draw_text(d, font, line.as_str(), x, y + i as i32 * line_height, size);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.lines[self.current_line].len() {
            self.cursor += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn next_line(&mut self) {
        if self.current_line < self.lines.len() {
            self.current_line += 1;
        }
        self.clamp_cursor();
    }

    pub fn prev_line(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
        }
        self.clamp_cursor();
    }

    pub fn start_of_line(&mut self) {
        self.cursor = 0;
    }

    pub fn end_of_line(&mut self) {
        self.cursor = self.current_line().len();
    }

    pub fn insert_char(&mut self, c: char) {
        self.lines[self.current_line].insert(self.cursor, c);
    }

    pub fn backspace(&mut self) {
        if self.lines[self.current_line].len() > self.cursor {
            self.lines[self.current_line].remove(self.cursor);
            self.clamp_cursor()
        }
    }

    pub fn clear(&mut self) {
        self.lines[self.current_line].clear();
        self.cursor = 0;
    }

    pub fn kill_to_end(&mut self) {
        self.lines[self.current_line].truncate(self.cursor);
    }

    pub fn current_line(&self) -> &str {
        self.lines[self.current_line].as_str()
    }

    pub fn randomise_line(&mut self) {
        self.clear();
        let mut rng = rand::rng();
        let sampled: [char; 8] = program::ALLOWED.choose_multiple_array(&mut rng).unwrap();
        self.lines[self.current_line] = sampled.iter().collect();
    }

    pub fn rotate_line_left(&mut self) {
        let mut s: Vec<char> = self.current_line().chars().collect();
        if s.is_empty() {
            return;
        }
        s.rotate_left(1);
        self.lines[self.current_line] = s.iter().collect();
    }

    pub fn rotate_line_right(&mut self) {
        let mut s: Vec<char> = self.current_line().chars().collect();
        if s.is_empty() {
            return;
        }
        s.rotate_right(1);
        self.lines[self.current_line] = s.iter().collect();
    }

    fn clamp_cursor(&mut self) {
        if self.cursor > self.lines[self.current_line].len() {
            self.cursor = self.lines[self.current_line].len();
        }
    }

    fn draw_cursor(&self, d: &mut RaylibDrawHandle, font: &Font, x: i32, y: i32, size: i32) {
        let em = font.measure_text("M", size as f32, 1.0);
        let line_height = em.y as i32;
        let cursor_width = em.x as i32;

        let x_offset = font
            .measure_text(&self.current_line()[..self.cursor], size as f32, 1.0)
            .x as i32;
        d.draw_rectangle_gradient_h(
            x + x_offset,
            y + self.current_line as i32 * line_height,
            cursor_width,
            line_height,
            Color::NAVAJOWHITE,
            Color::NAVAJOWHITE.alpha(0.0),
        );
    }
}
