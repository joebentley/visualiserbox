use std::collections::HashSet;

use crate::program;
use crate::utils;
use raylib::core::color::Color;

struct FadeAnimation {}

impl FadeAnimation {
    fn execute(
        &self,
        current_program: String,
        next_program: String,
        initial_values: [f32; 3],
        f: f32,
    ) -> Color {
        let c1 = program::execute_string_to_color(current_program.as_str(), initial_values);
        let c2 = program::execute_string_to_color(next_program.as_str(), initial_values);
        c1.lerp(c2, f)
    }
}

struct DissolveAnimation {
    width: i32,
    height: i32,
    pixels_seen: HashSet<i32>,
    pixels_needed_per_second: u32,
}

impl DissolveAnimation {
    pub fn new(width: i32, height: i32, should_finish_in_time: f32) -> Self {
        let pixels_needed_per_second =
            ((width * height) as f32 / should_finish_in_time).round() as u32;
        Self {
            width,
            height,
            pixels_seen: HashSet::new(),
            pixels_needed_per_second,
        }
    }

    fn execute(
        &self,
        current_program: String,
        next_program: String,
        x: i32,
        y: i32,
        t: f32,
    ) -> Color {
        let initial_values = [x as f32, y as f32, t];

        let i = y * self.width + x;

        if self.pixels_seen.contains(&i) {
            program::execute_string_to_color(next_program.as_str(), initial_values)
        } else {
            program::execute_string_to_color(current_program.as_str(), initial_values)
        }
    }

    fn tick(&mut self, frame_time: f32) {
        let num_pixels_to_mark_seen =
            (frame_time * self.pixels_needed_per_second as f32).round() as usize;
        for _ in 0..num_pixels_to_mark_seen {
            if self.pixels_seen.len() == (self.width * self.height) as usize {
                return;
            }
            let mut i = -1;
            while self.pixels_seen.contains(&i) {
                i = rand::random_range(0..self.width * self.height);
            }
            self.pixels_seen.insert(i);
        }
    }

    fn reset(&mut self) {
        self.pixels_seen.clear();
    }
}

fn number_of_changes_needed(from: String, to: String) -> u32 {
    let from_vec: Vec<u8> = from.into_bytes();
    let to_vec: Vec<u8> = to.into_bytes();
    let mut num_changes = (from_vec.len() as i32 - to_vec.len() as i32).unsigned_abs();

    for i in 0..from_vec.len().min(to_vec.len()) {
        if from_vec[i] != to_vec[i] {
            num_changes += 1;
        }
    }

    num_changes
}

fn make_one_change(from: String, to: String) -> String {
    let mut from_vec: Vec<u8> = from.into_bytes();
    let mut to_vec: Vec<u8> = to.into_bytes();

    if from_vec.len() > to_vec.len() {
        from_vec.pop();
    } else if to_vec.len() > from_vec.len() {
        to_vec.pop();
    } else if from_vec != to_vec {
        let mut i = 0;
        while from_vec[i] == to_vec[i] {
            i += 1;
        }
        from_vec[i] = to_vec[i];
    }

    String::from_utf8(from_vec).unwrap()
}

struct ProgramDissolveAnimation {
    current: String,
    next_program: String,
    should_finish_in_time: f32,
    time_between_each_change: f32,
    t: f32,
}

impl ProgramDissolveAnimation {
    fn new(should_finish_in_time: f32) -> Self {
        Self {
            current: String::new(),
            next_program: String::new(),
            should_finish_in_time,
            time_between_each_change: 0.0,
            t: 0.0,
        }
    }

    fn execute(
        &mut self,
        current_program: String,
        next_program: String,
        x: i32,
        y: i32,
        t: f32,
    ) -> Color {
        if self.current.is_empty() {
            self.set_new_programs(current_program, next_program);
        }

        let initial_values = [x as f32, y as f32, t];

        program::execute_string_to_color(&self.current, initial_values)
    }

    fn tick(&mut self, frame_time: f32) {
        if self.t >= self.time_between_each_change {
            self.current = make_one_change(self.current.clone(), self.next_program.clone());
            self.t = 0.0;
        }
        self.t += frame_time;
    }

    fn set_new_programs(&mut self, current_program: String, next_program: String) {
        self.time_between_each_change = self.should_finish_in_time
            / number_of_changes_needed(current_program.clone(), next_program.clone()) as f32;
        self.current = current_program;
        self.next_program = next_program;
    }

    fn reset(&mut self) {
        self.current.clear();
        self.t = 0.0;
    }
}

#[derive(PartialEq, Eq)]
enum Animation {
    Fade,
    Dissolve,
    ProgramDissolve,
}

impl Animation {
    fn random() -> Self {
        match rand::random_range(0..3) {
            0 => Animation::Fade,
            1 => Animation::Dissolve,
            _ => Animation::ProgramDissolve,
        }
    }
}

pub struct ProgramAnimator {
    playing: bool,
    fade_animation: FadeAnimation,
    dissolve_animation: DissolveAnimation,
    program_dissolve_animation: ProgramDissolveAnimation,
    current_animation: Animation,
    t: f32,
    cycle_time: f32,
    pause_fraction: f32,
}

impl ProgramAnimator {
    pub fn new(cycle_time: f32, pause_fraction: f32, width: i32, height: i32) -> Self {
        assert!(pause_fraction >= 0.0);
        assert!(pause_fraction < 1.0);
        let should_finish_in_time = cycle_time * (1.0 - pause_fraction);
        Self {
            playing: true,
            fade_animation: FadeAnimation {},
            dissolve_animation: DissolveAnimation::new(width, height, should_finish_in_time),
            program_dissolve_animation: ProgramDissolveAnimation::new(should_finish_in_time),
            current_animation: Animation::random(),
            t: 0.0,
            cycle_time,
            pause_fraction,
        }
    }

    pub fn execute(
        &mut self,
        current_program: String,
        next_program: String,
        x: i32,
        y: i32,
        t: f32,
    ) -> Color {
        if self.t <= self.pause_fraction {
            program::execute_string_to_color(current_program.as_str(), [x as f32, y as f32, t])
        } else {
            let f = utils::map(self.pause_fraction, 1.0, 0.0, 1.0, self.t);
            match self.current_animation {
                Animation::Fade => self.fade_animation.execute(
                    current_program,
                    next_program,
                    [x as f32, y as f32, t],
                    f,
                ),
                Animation::Dissolve => {
                    self.dissolve_animation
                        .execute(current_program, next_program, x, y, t)
                }
                Animation::ProgramDissolve => {
                    self.program_dissolve_animation
                        .execute(current_program, next_program, x, y, t)
                }
            }
        }
    }

    pub fn calculate_marker_y_position(
        &self,
        from_line: usize,
        to_line: usize,
        line_height: f32,
    ) -> f32 {
        utils::map(
            self.pause_fraction,
            1.0,
            from_line as f32 * line_height,
            to_line as f32 * line_height,
            self.t.clamp(self.pause_fraction, 1.0),
        )
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn play(&mut self) {
        self.reset();
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.reset();
        self.playing = false;
    }

    pub fn tick(&mut self, frame_time: f32) {
        if self.playing {
            self.t += frame_time / self.cycle_time;

            if self.t >= self.pause_fraction {
                match self.current_animation {
                    Animation::Dissolve => {
                        self.dissolve_animation.tick(frame_time);
                    }
                    Animation::ProgramDissolve => {
                        self.program_dissolve_animation.tick(frame_time);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn is_animation_finished(&self) -> bool {
        self.t >= 1.0
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.dissolve_animation.reset();
        self.program_dissolve_animation.reset();
        self.current_animation = Animation::random();
    }
}
