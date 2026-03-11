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
    measurements: Vec<(Pos, Vec<f32>)>,
    refinded_measurments: Vec<(Pos, Vec<f32>)>, // idee: Die measurments nehmen, und dann rückwürkend auch bearbeiten wenn es neue Infos gibt
    width: i32,
    height: i32,
    signal_radius: f32,
    signal_noise: f32,
    pub bot_pos: Pos,
    pub known_walls: HashSet<Pos>
}

impl SignalSolver {
    pub fn new(width: i32, height: i32, signal_radius: f32, bot_pos: Pos, signal_noise: f32, known_walls: HashSet<Pos>) -> Self {
        SignalSolver {
            width,
            height,
            refinded_measurments: Vec::new(),
            signal_radius,
            measurements: Vec::new(),
            signal_noise,
            bot_pos: bot_pos,
            known_walls: known_walls
        }
    }
    pub fn add_measurement(&mut self, channels: &Vec<f32>, pos: Pos) {
        if self.measurements.len() > 20 {
            self.measurements.remove(0);
        }
        
        self.measurements.push((pos, channels.clone()));
    }

    // man eigentlich ist das so geil
    //pub fn update_measurenments(&mut self, target_gem: Pos, radius: f32) {
    //    for (bot_pos, channels) in &mut self.measurements {
    //        let contrib = signal_from_dist(*bot_pos, target_gem, radius);
    //        *signal = (*signal - contrib).max(0.0);

    //    }
    //}

    pub fn clear_measuremnts(&mut self) {
        self.measurements.clear();
    }
    
    // berechnet wie gut die position zu dem gemessen signal wert passt und gibt einen score
    fn rate_position(&self, gem_positions: &[Pos], channel_num: usize) -> f32 {
        
        let mut total_error = 0.0;
        for (bot_pos, channel) in &self.measurements {
            let signal = channel[channel_num];
            let mut expected_signal = 0.0;
            for gem_pos in gem_positions {
                expected_signal += signal_from_dist(*gem_pos, *bot_pos, self.signal_radius);
            }

            // Bestimmter Fehler akzeptieren
            let error = (expected_signal - signal).abs();
            total_error += error;
        
            
        }
        
        1.0 / (0.001 + total_error)
    }

    pub fn find_single_gem(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channel_num: usize) -> Option<Pos> {
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

                let score = self.rate_position(&[pos], channel_num);
                
                if score > best_score {
                    best_score = score;
                    best_pos = Some(pos);
                }
            }
        }
        
        best_pos
    }


    pub fn get_test_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channels: &Vec<f32>) -> Option<Pos> {
        let mut highest_sig: f32 = 0.0;
        let mut best_channel: usize = 0;
        for i in 0..channels.len() {
            let sig = channels[i];
            if sig > highest_sig {
                highest_sig = sig;
                best_channel = i;
            }
        }

        eprintln!("{}", highest_sig);
        let target = self.find_single_gem(known_walls, vis_floors, best_channel);
        
        target
    }

    pub fn get_signal_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channel: &Vec<f32>) -> Option<Pos> {
        let mut positions: Vec<Pos> = Vec::new();

        for i in 0..channel.len() {
            if channel[i] == 0.0 {
                continue;
            }
            if let Some(pos) = self.find_single_gem(known_walls, vis_floors, i) {
                positions.push(pos)
            }
        }

        let mut best_dist: i32 = 0;
        let mut best: Pos = [0, 0];

        for i in positions {
            if let Some(dist) = get_dist(i, self.bot_pos, &self.known_walls, self.width, self.height) {
                if dist > best_dist {
                    best_dist = dist;
                    best = i;
                }
            }
        }
        Some(best)
    }
}

// calc the signal level from a euclidian distance
fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = euclidean(pos1, pos2);
    let s = 1.0 / (1.0 + (dist / radius).powf(2.0));

    let factor = 1_000_000.0_f32;
    (s * factor).trunc() / factor
}
