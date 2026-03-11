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
    width: i32,
    height: i32,
    signal_radius: f32,
    pub bot_pos:Pos
}

impl SignalSolver {
    pub fn new(width: i32, height: i32, signal_radius: f32, bot_pos: Pos) -> Self {
        SignalSolver {
            width,
            height,
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
        
        if let Some((coarse1, coarse2)) = best_pos {
            self.refine_positions(vec![coarse1, coarse2], vis_floors, known_walls)
                .map(|v| (v[0], v[1]))
        } else {
            None
        }
    }
    
    pub fn find_three_gems(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<(Pos, Pos, Pos)> {
        if self.measurements.is_empty() {
            return None;
        }
        
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
        
        if let Some((coarse1, coarse2, coarse3)) = best_pos {
            self.refine_positions(vec![coarse1, coarse2, coarse3], vis_floors, known_walls)
                .map(|v| (v[0], v[1], v[2]))
        } else {
            None
        }
    }
    
    // // immer ein bisschen verbessern => jeden gem um ein bisschen verschieben bis er besser passt (höherer score)
    // fn refine_positions(&self, mut positions: Vec<Pos>, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, reachable: &HashSet<Pos>) -> Option<Vec<Pos>> {
    //     let mut improved = true;
    //     let mut best_score = self.rate_position(&positions);
    //     let max_refinement_iterations = 200; 
    //     let mut iteration = 0;
        
    //     while improved && iteration < max_refinement_iterations {
    //         improved = false;
    //         iteration += 1;
            
    //         for gem_idx in 0..positions.len() {
    //             let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
    //             let old_pos = positions[gem_idx];
                
    //             for dir in &directions {
    //                 let new_pos = [old_pos[0] + dir[0], old_pos[1] + dir[1]];
                    
    //                 // Check bounds and validity
    //                 if new_pos[0] < 0 || new_pos[0] >= self.width || 
    //                    new_pos[1] < 0 || new_pos[1] >= self.height ||
    //                    known_walls.contains(&new_pos) || vis_floors.contains(&new_pos) {
    //                     continue;
    //                 }

    //                 if !reachable.contains(&new_pos) {
    //                     continue;
    //                 }
                    
    //                 positions[gem_idx] = new_pos;
    //                 let new_score = self.rate_position(&positions);
                    
    //                 if new_score > best_score {
    //                     best_score = new_score;
    //                     improved = true;
    //                     break; // Move to next gem once we found improvement
    //                 } else {
    //                     positions[gem_idx] = old_pos; // revert if it didnt improve
    //                 }
    //             }
    //         }
    //     }
        
    //     Some(positions)
    // }

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

    // pub fn get_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
    //     if self.measurements.len() < 3 {
    //         eprintln!("Not enough measurements yet");
    //         return None;
    //     }
        
    //     // try to search for one gem first
    //     let gem1 = self.find_single_gem(known_walls, vis_floors)?;
    //     let score1 = self.rate_position(&[gem1]);
        
    //     eprintln!("1-gem solution: {:?} with score {:.2}", gem1, score1);
        
    //     // wenn der eine gem gut ist dann ist es wahrscheinlich ein gem
    //     if score1 > 50.0 { // kann man tweaken
    //         eprintln!("High confidence: 1 gem");
    //         return Some(gem1);
    //     }
        
    //     // sonst nach 2 gems ausprobieren
    //     let (gem_a, gem_b) = self.find_two_gems(known_walls, vis_floors)?;
    //     let score2 = self.rate_position(&[gem_a, gem_b]);
        
    //     eprintln!("2-gem solution: {:?}, {:?} with score {:.2}", gem_a, gem_b, score2);
        
    //     // wenn 20 gems min. 50 % besser ist als 1 gem, dann 2 gems nehmen
    //     if score2 > score1 * 1.5 && score2 > 50.0 {
    //         eprintln!("High confidence: 2 gems");
    //         // Return closest gem to current bot position
    //         return Some(self.get_closest_gem(&[gem_a, gem_b]));
    //     }
        
    //     // wenn auch 2 gems schlecht sind, sind es wahrscheinlich 3 gems
    //     if score2 < 30.0 {
    //         let (gem_x, gem_y, gem_z) = self.find_three_gems(known_walls, vis_floors)?;
    //         let score3 = self.rate_position(&[gem_x, gem_y, gem_z]);
            
    //         eprintln!("3-gem solution: {:?}, {:?}, {:?} with score {:.2}", gem_x, gem_y, gem_z, score3);
            
    //         // wenn 3 gems 30 % besser als 2 gems istr, dann sind es 3
    //         if score3 > score2 * 1.3 {
    //             eprintln!("Using 3-gem solution");
    //             return Some(self.get_closest_gem(&[gem_x, gem_y, gem_z]));
    //         }
    //     }
        
    //     // default zurücl zur besten solution von beiden wenn es nicht geht
    //     if score2 > score1 {
    //         eprintln!("default: Using 2-gem solution");
    //         Some(self.get_closest_gem(&[gem_a, gem_b]))
    //     } else {
    //         eprintln!("default: Using 1-gem solution");
    //         Some(gem1)
    //     }
    // }

    pub fn get_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
        if self.measurements.len() < 3 {
            return None;
        }
        
        // try 1 gem
        let gem1 = self.find_single_gem(known_walls, vis_floors)?;
        let score1 = self.rate_position(&[gem1]);
        
        // try 2 gems
        let (gem_a, gem_b) = self.find_two_gems(known_walls, vis_floors)?;
        let score2 = self.rate_position(&[gem_a, gem_b]);
        
        // try 3 gems
        let (gem_x, gem_y, gem_z) = self.find_three_gems(known_walls, vis_floors)?;
        let score3 = self.rate_position(&[gem_x, gem_y, gem_z]);
        
        // pick the one with the highest score
        if score3 >= score2 && score3 >= score1 {
            eprintln!("3 gem solution: {:?}, {:?}, {:?} with score {:.2}", gem_x, gem_y, gem_z, score3);
            let gem = Some(self.get_closest_gem(&[gem_x, gem_y, gem_z]));
            eprint!("closest: {:?}", gem);
            gem
        } else if score2 >= score1 {
            eprintln!("2 gem solution: {:?}, {:?} with score {:.2}", gem_a, gem_b, score2);
            let gem = Some(self.get_closest_gem(&[gem_a, gem_b]));
            eprint!("closest: {:?}", gem);
            gem
        } else {
            eprintln!("1 gem solution: {:?} with score {:.2}", gem1, score1);
            Some(gem1)
        }
    }

    pub fn get_test_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) -> Option<Pos> {
        
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