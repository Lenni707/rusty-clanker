use std::collections::{HashMap, HashSet};

use crate::states::GameState;

pub struct BotMemory {
    pub current_target: Option<GemMemory>,
    pub known_gems: HashMap<[i32; 2], GemMemory>,
    pub known_positions: HashMap<[i32; 2], PositionMemory>, // ich weiß wirklich nicht wieso positions, das sind eigentlich die floors
    pub known_walls: HashSet<[i32; 2]>,
    pub exploration_target: Option<[i32; 2]>,
    // pub last_steps
}
#[derive(Debug, PartialEq)]
pub struct GemMemory {
    pub position: [i32; 2],
    pub ttl: i32,
}
#[derive(Debug, Clone, Copy)]
pub struct PositionMemory {
    pub last_seen_tick: u32,
    pub vis_floor_count: usize,
}
impl BotMemory {
    pub fn new() -> Self {
        BotMemory {
            current_target: None,
            known_gems: HashMap::new(),
            known_positions: HashMap::new(),
            known_walls: HashSet::new(),
            exploration_target: None,
        }
    }
    // Update: Wurden neue gems gesehen? Läuft vor decide
    pub fn update(&mut self, state: &GameState) {
        // Alle Gems Markieren
        for gem in &state.visible_gems {
            self.known_gems.insert(
                gem.position,
                GemMemory {
                    position: gem.position,
                    ttl: gem.ttl,
                },
            );
        }
        // Alle Sichbaren Positionen markieren
        let vis_radius = 5; // Positionen, welche nah an dem Bot sind werden als erkundet gezählt.
        for floor_pos in &state.floor {
            let distance = get_dist(state.bot, *floor_pos);
            if distance <= vis_radius * vis_radius {
                self.known_positions.insert(*floor_pos, PositionMemory {
                    last_seen_tick: state.tick,
                    vis_floor_count: state.floor.len(),
                });
            }
        }
        // Alle sichtbaren Walls markieren
        for wall_pos in &state.wall {
            self.known_walls.insert(*wall_pos);
        }

        // self.known_positions.retain(|_, &mut last_seen| state.tick - last_seen < 250);

        // Check für das current target
        if let Some(target) = &self.current_target {
            // Wenn es ein Target gibt: Dann nichtmehr zum Exploration Target zurück gehen
            self.exploration_target = None;
            // Checks für curr target
            let reached = state.bot == target.position;
            let mut vivsible = false;
            for gem in state.visible_gems.iter() {
                if gem.position == target.position {
                    vivsible = true;
                }
            }
            if reached && !vivsible {
                self.known_gems.remove(&target.position);
                self.current_target = None;
                eprintln!("Target not there, {} {}", state.tick, vivsible)
            }
        }

        // Exploratin Check: Wurde das Ziel erreicht?
        if let Some(explore_pos) = self.exploration_target {
            // Ist das Target erreicht?
            if state.bot == explore_pos {
                self.exploration_target = None;
            }
            // Maybe noch etwas machen, dass wenn es gesehen wurde das dann nicht weiter gegangen wird?
        }

        // Ziel wählen - always pick closest gem
        if !self.known_gems.is_empty() {
            if let Some(pos) = self.get_closest_gem(state) {
                if let Some(gem) = self.known_gems.get(&pos) {
                    // eprintln!("{:?}, {}", self.current_target, state.tick);
                    self.current_target = Some(GemMemory {
                        position: pos,
                        ttl: gem.ttl,
                    });
                }
            }
        }
    }

    pub fn get_target(&self) -> Option<[i32; 2]> {
        match &self.current_target {
            Some(gem) => Some(gem.position),
            None => None,
        }
    }

    pub fn get_closest_gem(&self, state: &GameState) -> Option<[i32; 2]> {
        self.known_gems
            .keys()
            .min_by_key(|pos| heuristic(**pos, state.bot))
            .copied()
    }
}

pub fn get_dist(a: [i32; 2], b: [i32; 2]) -> i32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
fn heuristic(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}
