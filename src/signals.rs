use crate::helper_func::{Pos, get_dist};
use crate::states::GameState;


pub struct SignalSolver {
    measurements: Vec<(Pos, f32)>,
    width: i32,
    height: i32,
    signal_radius: f32,
}

impl SignalSolver {
    pub fn new(width: i32, height: i32, signal_radius: f32) -> Self {
        SignalSolver {
            width,
            height,
            signal_radius,
            measurements: Vec::new(),
        }
    }
    pub fn add_measurement(&mut self, signal_level: f32, pos: Pos) {
        self.measurements.push((pos, signal_level));
    }
    fn predicted_score(&self, gem_pos: &Pos) -> f32 {
        let mut total_error = 0.0;
        for (bot_pos, signal) in &self.measurements {
            let mut expected_signal = 0.0;
            expected_signal += signal_from_dist(*gem_pos, *bot_pos, self.signal_radius);
        }
        expected_signal
    }

    pub fn find_single_gem() {}

    pub fn get_target() {}
}


fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = get_dist(a, b, memory)
    1.0 / (1.0 + (dist / radius).powf(2.0))
}