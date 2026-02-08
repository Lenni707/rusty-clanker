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
            let contrib = signal_from_dist(*bot_pos, target_gem, radius);
            *signal = (*signal - contrib).max(0.0);

        }
    }

    pub fn clear_measuremnts(&mut self) {
        self.measurements.clear();
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
        
        // Coarse search: every 5th cell
        let step = 5;
        let mut best_score = 0.0;
        let mut best_pos = None;
        
        for x1 in (0..self.width).step_by(step) {
            for y1 in (0..self.height).step_by(step) {
                let pos1 = [x1, y1];
                if known_walls.contains(&pos1) || vis_floors.contains(&pos1) {
                    continue;
                }
                
                for x2 in (0..self.width).step_by(step) {
                    for y2 in (0..self.height).step_by(step) {
                        let pos2 = [x2, y2];
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
        
        // Refine locally around best coarse solution
        if let Some((coarse1, coarse2)) = best_pos {
            self.refine_positions(vec![coarse1, coarse2], known_walls, vis_floors)
                .map(|v| (v[0], v[1]))
        } else {
            None
        }
    }
    
    pub fn find_three_gems(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<(Pos, Pos, Pos)> {
        if self.measurements.is_empty() {
            return None;
        }
        
        // Coarse search: every 6th cell for 3 gems
        let step = 6;
        let mut best_score = 0.0;
        let mut best_pos = None;
        
        for x1 in (0..self.width).step_by(step) {
            for y1 in (0..self.height).step_by(step) {
                let pos1 = [x1, y1];
                if known_walls.contains(&pos1) || vis_floors.contains(&pos1) {
                    continue;
                }
                
                for x2 in (0..self.width).step_by(step) {
                    for y2 in (0..self.height).step_by(step) {
                        let pos2 = [x2, y2];
                        if known_walls.contains(&pos2) || vis_floors.contains(&pos2) {
                            continue;
                        }
                        
                        for x3 in (0..self.width).step_by(step) {
                            for y3 in (0..self.height).step_by(step) {
                                let pos3 = [x3, y3];
                                if known_walls.contains(&pos3) || vis_floors.contains(&pos3) {
                                    continue;
                                }
                                
                                let score = self.rate_position(&[pos1, pos2, pos3]);
                                if score > best_score {
                                    best_score = score;
                                    best_pos = Some((pos1, pos2, pos3));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Refine locally around best coarse solution
        if let Some((coarse1, coarse2, coarse3)) = best_pos {
            self.refine_positions(vec![coarse1, coarse2, coarse3], known_walls, vis_floors)
                .map(|v| (v[0], v[1], v[2]))
        } else {
            None
        }
    }
    
    // Hill climbing refinement: move each gem to better adjacent positions
    fn refine_positions(&self, mut positions: Vec<Pos>, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Vec<Pos>> {
        let mut improved = true;
        let mut best_score = self.rate_position(&positions);
        let max_refinement_iterations = 100; // Prevent infinite loops
        let mut iteration = 0;
        
        while improved && iteration < max_refinement_iterations {
            improved = false;
            iteration += 1;
            
            // Try to improve each gem position
            for gem_idx in 0..positions.len() {
                let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
                let old_pos = positions[gem_idx];
                
                for dir in &directions {
                    let new_pos = [old_pos[0] + dir[0], old_pos[1] + dir[1]];
                    
                    // Check bounds and validity
                    if new_pos[0] < 0 || new_pos[0] >= self.width || 
                       new_pos[1] < 0 || new_pos[1] >= self.height ||
                       known_walls.contains(&new_pos) || vis_floors.contains(&new_pos) {
                        continue;
                    }
                    
                    positions[gem_idx] = new_pos;
                    let new_score = self.rate_position(&positions);
                    
                    if new_score > best_score {
                        best_score = new_score;
                        improved = true;
                        break; // Move to next gem once we found improvement
                    } else {
                        positions[gem_idx] = old_pos; // Revert if not better
                    }
                }
            }
        }
        
        Some(positions)
    }

    
    // pub fn get_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
    //     let gem = self.find_single_gem(known_walls, vis_floors)?;
    //     let score = self.rate_position(&[gem]);
    //     // if score < 500.0 {
    //     //     let gems = self.find_two_gems(known_walls, vis_floors)?;
            
    //     //     let score = self.rate_position(&[gems.0, gems.1]);

    //     //     return Some(gems.0);
    //     // }
    //     eprintln!("Found gem at {:?} with score {}", gem, score);
    //     Some(gem)
    // }

    pub fn get_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
        if self.measurements.len() < 3 {
            eprintln!("Not enough measurements yet");
            return None;
        }
        
        // Try 1 gem first (always fastest)
        let gem1 = self.find_single_gem(known_walls, vis_floors)?;
        let score1 = self.rate_position(&[gem1]);
        
        eprintln!("1-gem solution: {:?} with score {:.2}", gem1, score1);
        
        // If score is very good, probably just 1 gem
        if score1 > 50.0 {
            eprintln!("High confidence: 1 gem");
            return Some(gem1);
        }
        
        // Try 2 gems
        let (gem_a, gem_b) = self.find_two_gems(known_walls, vis_floors)?;
        let score2 = self.rate_position(&[gem_a, gem_b]);
        
        eprintln!("2-gem solution: {:?}, {:?} with score {:.2}", gem_a, gem_b, score2);
        
        // Check if 2-gem is significantly better (at least 50% improvement)
        if score2 > score1 * 1.5 && score2 > 50.0 {
            eprintln!("High confidence: 2 gems");
            // Return closest gem to current bot position
            return Some(self.get_closest_gem(&[gem_a, gem_b]));
        }
        
        // If 2-gem wasn't much better, might be 3 gems
        if score2 < 30.0 {
            let (gem_x, gem_y, gem_z) = self.find_three_gems(known_walls, vis_floors)?;
            let score3 = self.rate_position(&[gem_x, gem_y, gem_z]);
            
            eprintln!("3-gem solution: {:?}, {:?}, {:?} with score {:.2}", gem_x, gem_y, gem_z, score3);
            
            // Use 3-gem if significantly better than 2-gem
            if score3 > score2 * 1.3 {
                eprintln!("Using 3-gem solution");
                return Some(self.get_closest_gem(&[gem_x, gem_y, gem_z]));
            }
        }
        
        // Default to best solution found
        if score2 > score1 {
            eprintln!("Using 2-gem solution");
            Some(self.get_closest_gem(&[gem_a, gem_b]))
        } else {
            eprintln!("Using 1-gem solution");
            Some(gem1)
        }
    }

    // Helper: find closest gem to bot's most recent position
    fn get_closest_gem(&self, gems: &[Pos]) -> Pos {
        if let Some((last_bot_pos, _)) = self.measurements.last() {
            gems.iter()
                .min_by_key(|gem| {
                    let dx = gem[0] - last_bot_pos[0];
                    let dy = gem[1] - last_bot_pos[1];
                    dx * dx + dy * dy
                })
                .copied()
                .unwrap_or(gems[0])
        } else {
            gems[0]
        }
    }
}



fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = euclidean(pos1, pos2);
    let s = 1.0 / (1.0 + (dist / radius).powf(2.0));

    let factor = 1_000_000.0_f32;
    (s * factor).trunc() / factor
}
