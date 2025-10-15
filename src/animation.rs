use crate::program;
use crate::utils;

pub struct ProgramAnimator {
    t: f32,
    cycle_time: f32,
    pause_fraction: f32,
    playing: bool,
}

impl ProgramAnimator {
    pub fn new(cycle_time: f32, pause_fraction: f32) -> Self {
        assert!(pause_fraction >= 0.0);
        assert!(pause_fraction < 1.0);
        Self {
            t: 0.0,
            cycle_time,
            pause_fraction,
            playing: true,
        }
    }

    pub fn get_blend_mode(
        &self,
        current_program: impl Into<String>,
        next_program: impl Into<String>,
    ) -> program::BlendMode {
        if self.t < self.pause_fraction {
            program::BlendMode::One(current_program.into())
        } else {
            program::BlendMode::Two(
                current_program.into(),
                next_program.into(),
                utils::map(self.pause_fraction, 1.0, 0.0, 1.0, self.t),
            )
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
        }
    }

    pub fn needs_program(&self) -> bool {
        self.t >= 1.0
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
    }
}
