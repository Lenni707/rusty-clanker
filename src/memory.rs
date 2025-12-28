use std::collections::HashMap;

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

        // Neues Ziel wählen
        if self.current_target.is_none() && !self.known_gems.is_empty() {
            // Wenn es aktuell kein Ziel gibt, aber uns noch ein gem bekannst ist:
            if let Some((&pos, gem)) = self.known_gems.iter().next() {
                self.current_target = Some(
                    GemMemory { 
                        position: pos,
                        ttl: gem.ttl 
                    }
                )
            }
        }
    }

    pub fn get_target(&self) -> Option<[i32; 2]> {
        match &self.current_target {
            Some(gem) => Some(gem.position),
            None => None
        }
    }
}
