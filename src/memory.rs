use std::{collections::HashMap, string};

use crate::states::GameState;

pub struct BotMemory {
    pub current_target: Option<GemMemory>,
    pub known_gems: HashMap<[i32; 2], GemMemory>,
    // pub explored_position: HashSet<[i32; 2]> // später für eine Logik, wenn kein Gem zu sehen ist
}
#[derive(Debug)]
pub struct GemMemory {
    pub position: [i32; 2],
    pub ttl: i32,
}

impl BotMemory {
    pub fn new() -> Self {
        BotMemory {
            current_target: None,
            known_gems: HashMap::new(),
        }
    }
    // Update: Wurden neue gems gesehen? Läuft vor decide
    pub fn update(&mut self, state: &GameState) {
        for gem in &state.visible_gems {
            self.known_gems.insert(
                gem.position,
                GemMemory { 
                    position: gem.position, 
                    ttl: gem.ttl
                }
            );
        }
        // Check für das current target
        if let Some(target) = &self.current_target {
            let reached = state.bot == target.position;
            // TODO: Hier vielleicht noch einen check: Ist das target überhaupt sichbar?
            let mut vivsible = false;
            for gem in state.visible_gems.iter() {
                if gem.position == target.position {
                    vivsible = true;
                }
            }
            if reached || !vivsible {
                self.known_gems.remove(&target.position);
                self.current_target = None;
            }
        }

        // Ziel wählen - always pick closest gem
        if !self.known_gems.is_empty() {
            if let Some(pos) = self.get_closest_gem(state) {
                if let Some(gem) = self.known_gems.get(&pos) {
                    self.current_target = Some(
                        GemMemory { 
                            position: pos,
                            ttl: gem.ttl 
                        }
                    );
                }
            }
        }
    }

    pub fn get_target(&self) -> Option<[i32; 2]> {
        match &self.current_target {
            Some(gem) => Some(gem.position),
            None => None
        }
    }

    pub fn get_closest_gem(&self, state: &GameState) -> Option<[i32; 2]> {
        let closest = self.known_gems
            .keys()
            .reduce(|k1, k2| { // folds the iterator by applying the get_dist function
                if get_dist(*k1, state.bot) <= get_dist(*k2, state.bot) {
                    k1
                } else {
                    k2
                }
            })
            .copied() // bc its a pointer
            .unwrap(); // extracts from Option type
        Some(closest)
    }
}

pub fn get_dist(a: [i32; 2], b: [i32; 2]) -> i32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
