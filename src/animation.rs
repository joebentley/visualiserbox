use crate::program;
use crate::utils;
use raylib::core::color::Color;

struct FadeAnimation {
    cycle_time: f32,
    pause_fraction: f32,
    t: f32,
}

impl FadeAnimation {
    pub fn new(cycle_time: f32, pause_fraction: f32) -> Self {
        Self {
            t: 0.0,
            cycle_time,
            pause_fraction,
        }
    }

    fn tick(&mut self, frame_time: f32) {
        self.t += frame_time / self.cycle_time;
    }

    fn execute(
        &self,
        current_program: String,
        next_program: String,
        initial_values: [f32; 3],
    ) -> Color {
        if self.t <= self.pause_fraction {
            program::execute_string_to_color(current_program.as_str(), initial_values)
        } else {
            let f = utils::map(self.pause_fraction, 1.0, 0.0, 1.0, self.t);
            let c1 = program::execute_string_to_color(current_program.as_str(), initial_values);
            let c2 = program::execute_string_to_color(next_program.as_str(), initial_values);
            c1.lerp(c2, f)
        }
    }

    fn is_finished(&self) -> bool {
        self.t >= 1.0
    }

    fn reset(&mut self) {
        self.t = 0.0;
    }
}

enum Animation {
    Fade,
}

pub struct ProgramAnimator {
    playing: bool,
    fade_animation: FadeAnimation,
    current_animation: Animation,
    t: f32,
    cycle_time: f32,
    pause_fraction: f32,
}

impl ProgramAnimator {
    pub fn new(cycle_time: f32, pause_fraction: f32) -> Self {
        assert!(pause_fraction >= 0.0);
        assert!(pause_fraction < 1.0);
        let mut me = Self {
            playing: true,
            fade_animation: FadeAnimation::new(cycle_time, pause_fraction),
            current_animation: Animation::Fade,
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
        initial_values: [f32; 3],
    ) -> Color {
        match self.current_animation {
            Animation::Fade => {
                self.fade_animation
                    .execute(current_program, next_program, initial_values)
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
            self.fade_animation.tick(frame_time);
            self.t += frame_time / self.cycle_time;
        }
    }

    pub fn is_animation_finished(&self) -> bool {
        match self.current_animation {
            Animation::Fade => self.fade_animation.is_finished(),
        }
    }

    pub fn reset(&mut self) {
        self.fade_animation.reset();
        self.t = 0.0;
    }
}
