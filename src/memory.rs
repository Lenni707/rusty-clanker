use std::collections::{BTreeMap, HashSet};

use crate::states::GameState;

use crate::decision::astar;

pub struct BotMemory {
    pub current_target: Option<GemMemory>,
    pub known_gems: BTreeMap<[i32; 2], GemMemory>,
    pub known_positions: BTreeMap<[i32; 2], PositionMemory>, // ich weiß wirklich nicht wieso positions, das sind eigentlich die floors
    pub known_walls: HashSet<[i32; 2]>,
    pub exploration_target: Option<[i32; 2]>,
    pub gem_spawn_heatmap: BTreeMap<[i32; 2], u32>, // pub last_steps
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
            known_gems: BTreeMap::new(),
            known_positions: BTreeMap::new(),
            known_walls: HashSet::new(),
            exploration_target: None,
            gem_spawn_heatmap: BTreeMap::new(),
        }
    }
    // Update: Wurden neue gems gesehen? Läuft vor decide
    pub fn update(&mut self, state: &GameState) {
        // Alle Gems Markieren
        for gem in &state.visible_gems {
            // Gems spawns in die Heatmap eintragen,
            *self.gem_spawn_heatmap.entry(gem.position).or_insert(0) += 1;
            self.known_gems.insert(
                gem.position,
                GemMemory {
                    position: gem.position,
                    ttl: gem.ttl,
                },
            );
        }
        // Alle Sichbaren Positionen markieren
        // let vis_radius = 5; // Positionen, welche nah an dem Bot sind werden als erkundet gezählt.
        for floor_pos in &state.floor {
            self.known_positions.insert(
                *floor_pos,
                PositionMemory {
                    last_seen_tick: state.tick,
                    vis_floor_count: state.floor.len(),
                },
            );
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
                    break;
                }
            }
            if reached || (!vivsible && state.floor.contains(&target.position)) {
                // TODO: Das ist komisch hier, aber wenn es ein ODER ist dann geht es kaputt
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
        let mut gems: Vec<_> = self.known_gems.keys().copied().collect();
        // Wenn gems leer sind
        if gems.len() < 1 { return None; }
        gems.sort_by_key(|&[x, y]| (x, y)); // Sortiere deterministisch

        let mut closest: Option<[i32; 2]> = None;

        for gem in gems {
            closest = match closest {
                None => Some(gem),
                Some(current_closest) => {
                    match get_closer_pos(state.bot, current_closest, gem, self, &state) {
                        Some(closer) => Some(closer),
                        None => Some(current_closest)
                    }
                }
            }
        }

        closest
    }
}

pub fn get_dist(a: [i32; 2], b: [i32; 2]) -> i32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
fn _heuristic(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}

pub fn get_closer_pos(
    goal: [i32; 2],
    a: [i32; 2],
    b: [i32; 2],
    memory: &BotMemory,
    state: &GameState,
) -> Option<[i32; 2]> {
    if let Some(config) = &state.config {  
        let path_a = astar(a, goal, &memory.known_walls, config.width, config.height);
        let path_b = astar(b, goal, &memory.known_walls, config.width, config.height);

        let shorter = match (path_a, path_b) {
            (Some(pa), Some(pb)) => {
                if pa.len() <= pb.len() {
                    Some(a)
                } else {
                    Some(b)
                }
            }
            (Some(_), None) => Some(a),
            (None, Some(_)) => Some(b),
            (None, None) => { eprintln!("both positions cant be reached"); None }
        };
        shorter
    } else { return None }
    
}
