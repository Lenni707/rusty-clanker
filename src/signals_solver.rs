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

        // Schätze Anzahl der Gems basierend auf durchschnittlichem Signal
        let avg_signal: f32 = self.measurements.iter().map(|(_, s)| s).sum::<f32>() 
            / self.measurements.len() as f32;
        
        let max_signal: f32 = self.measurements.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        
        eprintln!("Avg signal: {:.3}, Max signal: {:.3}", avg_signal, max_signal);
        
        // Einfache Heuristik: Bei hohem Signal -> wahrscheinlich mehrere Gems
        if avg_signal > 1.2 || max_signal > 2.5 {
            // Versuche Multiple Gems zu finden
            eprintln!("Trying multi-gem detection...");
            let gems = self.find_multiple_gems(known_walls, 3);
            
            if gems.len() > 1 {
                eprintln!("Found {} gems: {:?}", gems.len(), gems);
                // Gib das erste Gem zurück (kannst auch closest zum Bot wählen)
                return Some(gems[0]);
            }
        }
        
        // Fallback: Single Gem
        eprintln!("Using single gem detection...");
        self.find_single_gem(known_walls)
    }

    pub fn find_multiple_gems(&self, known_walls: &HashSet<Pos>, max_gems: usize) -> Vec<Pos> {
        let mut found_gems = Vec::new();
        let mut remaining_measurements = self.measurements.clone();
        
        for _ in 0..max_gems {
            if remaining_measurements.is_empty() {
                break;
            }
            
            // Finde bestes Gem für verbleibende Messungen
            let temp_solver = SignalSolver {
                width: self.width,
                height: self.height,
                signal_radius: self.signal_radius,
                measurements: remaining_measurements.clone(),
            };
            
            if let Some(gem_pos) = temp_solver.find_single_gem(known_walls) {
                found_gems.push(gem_pos);
                
                // Subtrahiere Signal dieses Gems von allen Messungen
                for (bot_pos, signal) in &mut remaining_measurements {
                    let gem_signal = signal_from_dist(gem_pos, *bot_pos, self.signal_radius);
                    *signal = (*signal - gem_signal).max(0.0);
                }
                
                // Entferne Messungen mit sehr niedrigem Signal
                remaining_measurements.retain(|(_, s)| *s > 0.05);
            } else {
                break;
            }
        }
        
        found_gems
    }
}

fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dx = (pos1[0] - pos2[0]) as f32;
    let dy = (pos1[1] - pos2[1]) as f32;
    let dist = (dx*dx + dy*dy).sqrt();
    1.0 / (1.0 + (dist / radius).powf(2.0))
}