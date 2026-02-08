use crate::helper_func::{Pos, euclidean, get_dist, heuristic};
use crate::memory::BotMemory;
use crate::states::GameState;

use std::collections::{HashMap, HashSet};

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
        if self.measurements.len() > 20 {
            self.measurements.remove(0);
        }
        
        self.measurements.push((pos, signal_level));
    }

    pub fn update_measurenments(&mut self, target_gem: Pos, radius: f32) {
        for (bot_pos, signal) in &mut self.measurements {
            *signal -= signal_from_dist(*bot_pos, target_gem, radius).max(0.0);
        }
    }
    
    // Berechnet, wie gut eine Gegebene position zu den letzen measurements passt
    fn rate_position(&self, gem_positions: &[Pos]) -> f32 {
        let mut total_error = 0.0;
        
        for (bot_pos, signal) in &self.measurements {
            let mut expected_signal = 0.0;
            for gem_pos in gem_positions {
                expected_signal += signal_from_dist(*gem_pos, *bot_pos, self.signal_radius);
            }
            let error = (expected_signal - signal).abs();
            total_error += error;
        }
        
        1.0 / (0.001 + total_error)
    }

    pub fn find_single_gem(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut best_score = 0.0;
        let mut best_pos = None;

        for x in 0..self.width {
            for y in 0..self.height {
                let pos = [x, y];

                if known_walls.contains(&pos) || vis_floors.contains(&pos) {
                    continue;
                }

                let score = self.rate_position(&[pos]);
                
                if score > best_score {
                    best_score = score;
                    best_pos = Some(pos);
                }
            }
        }
        
        best_pos
    }
    pub fn find_two_gems(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<(Pos, Pos)> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut best_score = 0.0;
        let mut best_pos = None;

        for x1 in 0..self.width {
            for y1 in 0..self.height {
                for x2 in 0..self.width {
                    for y2 in 0..self.height {
                        let pos1 = [x1, y1];
                        let pos2 = [x2, y2];
                        
                        if known_walls.contains(&pos1) || vis_floors.contains(&pos1) {
                            continue;
                        }
                        if known_walls.contains(&pos2) || vis_floors.contains(&pos2) {
                            continue;
                        }

                        let score = self.rate_position(&[pos1, pos2]);
                        if score > best_score {
                            best_score = score;
                            best_pos = Some((pos1, pos2));
                        }
                    }
                }
            }
        }
        best_pos
    }
    
    pub fn get_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
        let gem = self.find_single_gem(known_walls, vis_floors)?;
        let score = self.rate_position(&[gem]);
        eprintln!("Found gem at {:?} with score {}", gem, score);
        Some(gem)
    }
}



fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = euclidean(pos1, pos2);
    let s = 1.0 / (1.0 + (dist / radius).powf(2.0));

    let factor = 1_000_000.0_f32;
    (s * factor).trunc() / factor
}
