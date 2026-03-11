use crate::helper_func::{Pos, euclidean, get_dist, heuristic};
use crate::astar::astar;

use std::collections::{HashMap, HashSet};

// Meine Idee wie ich das machen würde: 
// Wir lassen die Grundstuktur hier wie sie ist, wir brauchen ja aber an sich nur noch die "find_single_gem", denn:
// - Wir haben die channels für jeden Gem. Jeden Frame gehen wir alle channels durch und gucken wie sicher wir uns sind, das wir den 
//   gem in diesem Channel gefunden haben.
// - Wir nehmen uns den Gem, bei dem wir uns am sichersten sind und der nah dran ist
//      - Theoretisch können wir auch noch einen extra check machen mit dem gesamten Signal-level, vielleicht wenn es sehr hoch ist oder so
// Das einzige was wir eigentlich machen müssen ist einen Filter einbauen, da es ja das signal noise gibt. Wir wissen wie stark das signal noise ist,
// damit können wir errechnen wie stark das noise ungefiltern sein müsste, und damit ungefähre messungen machen.

// In SigSolcer würde ich measurements zu einem Vec<Pos, Vec<f32>> machen, dann dadurch durchgehen. Das ist eigentlich machbar glaube ich.

// Generell sollten wir aber wenn wir lust haben mal ein bisschen was erneuern, was ist den bitte decide.rs???? Und memory ist auch grauenhaft!

pub struct SignalSolver {
    measurements: Vec<(Pos, f32)>,
    refinded_measurments: Vec<(Pos, Vec<f32>)>, // idee: Die measurments nehmen, und dann rückwürkend auch bearbeiten wenn es neue Infos gibt
    width: i32,
    height: i32,
    signal_radius: f32,
    pub bot_pos: Pos,
}

impl SignalSolver {
    pub fn new(width: i32, height: i32, signal_radius: f32, bot_pos: Pos) -> Self {
        SignalSolver {
            width,
            height,
            refinded_measurments: Vec::new(),
            signal_radius,
            measurements: Vec::new(),
            bot_pos: bot_pos
        }
    }
    pub fn add_measurement(&mut self, signal_level: f32, pos: Pos) {
        if self.measurements.len() > 20 {
            self.measurements.remove(0);
        }
        
        self.measurements.push((pos, signal_level));
    }

    // man eigentlich ist das so geil
    pub fn update_measurenments(&mut self, target_gem: Pos, radius: f32) {
        for (bot_pos, signal) in &mut self.measurements {
            let contrib = signal_from_dist(*bot_pos, target_gem, radius);
            *signal = (*signal - contrib).max(0.0);

        }
    }

    pub fn clear_measuremnts(&mut self) {
        self.measurements.clear();
    }
    
    // berechnet wie gut die position zu dem gemessen signal wert passt und gibt einen score
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

    pub fn find_single_gem(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channels: &Vec<f32>) -> Option<Pos> {
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
    
    
    // // immer ein bisschen verbessern => jeden gem um ein bisschen verschieben bis er besser passt (höherer score)

    fn refine_positions(&self, mut positions: Vec<Pos>, vis_floors: &Vec<Pos>, known_walls: &HashSet<Pos>) -> Option<Vec<Pos>> {
        let mut best_score = self.rate_position(&positions);
        let max_refinement_iterations = 1000;
        let mut iteration = 0;
        
        loop {
            if iteration >= max_refinement_iterations {
                break;
            }
            
            iteration += 1;
            let mut any_improvement = false;
            
            for gem_idx in 0..positions.len() {
                let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
                let old_pos = positions[gem_idx];
                let mut local_best_score = best_score;
                let mut local_best_pos = old_pos;
                
                for dir in &directions {
                    let new_pos = [old_pos[0] + dir[0], old_pos[1] + dir[1]];
                    
                    if vis_floors.contains(&new_pos) || known_walls.contains(&new_pos) { // new_pos[0] < 0 || new_pos[0] >= self.width || new_pos[1] < 0 || new_pos[1] >= self.height || known_walls.contains(&new_pos) || 
                        continue;
                    }
                    
                    positions[gem_idx] = new_pos;
                    let new_score = self.rate_position(&positions);
                    
                    if new_score >= local_best_score {
                        local_best_score = new_score;
                        local_best_pos = new_pos;
                    }
                    
                    positions[gem_idx] = old_pos;
                }
                
                // apply best move
                if local_best_pos != old_pos {
                    positions[gem_idx] = local_best_pos;
                    best_score = local_best_score;  // Update global best
                    any_improvement = true;
                }
            }
            
            // Stop if no gem improved in this iteration
            if !any_improvement {
                eprintln!("refining stopped after {} iterations with score {:.2}", iteration, best_score);
                break;
            }
        }
        
        Some(positions)
    }


    pub fn get_test_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channels: &Vec<f32>) -> Option<Pos> {
        let mut highest_sig: f32 = 0.0;
        for sig in channels {
            if sig > &highest_sig {
                highest_sig = *sig;
            }
        }
        let target = self.find_single_gem(known_walls, vis_floors, channels);
        
        target
    }

    // find the closest gem
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

// calc the signal level from a euclidian distance
fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = euclidean(pos1, pos2);
    let s = 1.0 / (1.0 + (dist / radius).powf(2.0));

    let factor = 1_000_000.0_f32;
    (s * factor).trunc() / factor
}