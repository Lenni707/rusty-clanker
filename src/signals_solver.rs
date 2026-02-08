use std::collections::HashSet;
use crate::helper_func::{Pos, heuristic};

pub struct SignalSolver {
    width: i32,
    height: i32,
    signal_radius: f32,
    measurements: Vec<(Pos, f32)>,
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

    pub fn add_measurement(&mut self, bot_pos: Pos, signal: f32) {
        self.measurements.push((bot_pos, signal));
        
        // Behalte nur letzte 20 Messungen
        if self.measurements.len() > 20 {
            self.measurements.remove(0);
        }
    }

    fn score_hypothesis(&self, gem_positions: &[Pos]) -> f32 {
        let mut total_error = 0.0;
        
        for (bot_pos, observed_signal) in &self.measurements {
            let mut expected_signal = 0.0;
            for gem_pos in gem_positions {
                expected_signal += signal_from_dist(*gem_pos, *bot_pos, self.signal_radius);
            }
            
            let error = (expected_signal - observed_signal).abs();
            total_error += error;
        }
        
        1.0 / (0.001 + total_error)
    }

    pub fn find_single_gem(&self, known_walls: &HashSet<Pos>) -> Option<Pos> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut best_pos = None;
        let mut best_score = 0.0;

        for x in 0..self.width {
            for y in 0..self.height {
                let pos = [x, y];
                
                if known_walls.contains(&pos) {
                    continue;
                }

                let score = self.score_hypothesis(&[pos]);
                
                if score > best_score {
                    best_score = score;
                    best_pos = Some(pos);
                }
            }
        }

        eprintln!("Best single gem score: {}", best_score);
        best_pos
    }

    pub fn get_target(&self, known_walls: &HashSet<Pos>) -> Option<Pos> {
        if self.measurements.len() < 3 {
            return None;
        }

        self.find_single_gem(known_walls)
    }
}

fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dx = (pos1[0] - pos2[0]) as f32;
    let dy = (pos1[1] - pos2[1]) as f32;
    let dist = (dx*dx + dy*dy).sqrt();
    1.0 / (1.0 + (dist / radius).powf(2.0))
}