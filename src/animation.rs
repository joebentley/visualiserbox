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

#[derive(PartialEq, Eq)]
enum Animation {
    Fade,
    Dissolve,
}

impl Animation {
    fn random() -> Self {
        match rand::random_range(0..2) {
            0 => Animation::Fade,
            _ => Animation::Dissolve,
        }
    }
}

pub struct ProgramAnimator {
    playing: bool,
    fade_animation: FadeAnimation,
    dissolve_animation: DissolveAnimation,
    current_animation: Animation,
    t: f32,
    cycle_time: f32,
    pause_fraction: f32,
}

impl ProgramAnimator {
    pub fn new(cycle_time: f32, pause_fraction: f32, width: i32, height: i32) -> Self {
        assert!(pause_fraction >= 0.0);
        assert!(pause_fraction < 1.0);
        let mut me = Self {
            playing: true,
            fade_animation: FadeAnimation {},
            dissolve_animation: DissolveAnimation::new(
                width,
                height,
                cycle_time * (1.0 - pause_fraction),
            ),
            current_animation: Animation::random(),
            t: 0.0,
            cycle_time,
            pause_fraction,
        };
        me.reset();
        me
    }

    pub fn execute(
        &self,
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

            if self.t >= self.pause_fraction && self.current_animation == Animation::Dissolve {
                self.dissolve_animation.tick(frame_time);
            }
        }
    }

    pub fn is_animation_finished(&self) -> bool {
        self.t >= 1.0
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.dissolve_animation.reset();
        self.current_animation = Animation::random();
    }
}
