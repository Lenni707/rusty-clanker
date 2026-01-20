use std::collections::{BTreeMap, HashSet};
use std::collections::VecDeque; // iwie effiezient oder so ka
use crate::states::GameState;

use crate::helper_func::{get_dist, Pos};

pub struct BotMemory {
    pub config: Config,

    pub current_target: Option<GemMemory>,
    pub known_gems: BTreeMap<[i32; 2], GemMemory>,
    pub known_positions: BTreeMap<[i32; 2], PositionMemory>, // ich weiß wirklich nicht wieso positions, das sind eigentlich die floors
    pub known_walls: HashSet<[i32; 2]>,
    pub exploration_target: Option<[i32; 2]>,
    pub gem_spawn_heatmap: BTreeMap<[i32; 2], u32>, // pub last_steps

    pub signal_target: Option<Pos>,
    pub gem_signal_map: BTreeMap<[i32; 2], f32>,
    pub last_pos: Option<Pos>,
    pub last_signal: Option<f32>,
    pub recent_positions: VecDeque<Pos>,
    pub ticks_since_signal_up: i32
}

pub struct Config {
    pub width: i32,
    pub height: i32,
    pub vis_radius: i32,
    pub max_ticks: u32,
    pub gem_ttl: u32,
    pub signal_radius: f32,
}

#[derive(Debug, PartialEq)]
pub struct GemMemory {
    pub position: [i32; 2],
    pub ttl: i32,
    last_seen_tick: u32
}

#[derive(Debug, Clone, Copy)]
pub struct PositionMemory {
    pub last_seen_tick: u32,
    pub vis_floor_count: usize,
}

impl Config {
    pub fn new() -> Self {
        Config {
            width: 0,
            height: 0,
            vis_radius: 0,
            max_ticks: 0,
            gem_ttl: 0,
            signal_radius: 0.0,
        }
    }
}

impl BotMemory {
    pub fn new() -> Self {
        BotMemory {
            config: Config::new(),

            current_target: None,
            known_gems: BTreeMap::new(),
            known_positions: BTreeMap::new(),
            known_walls: HashSet::new(),
            exploration_target: None,
            gem_spawn_heatmap: BTreeMap::new(),

            signal_target: None,
            gem_signal_map: BTreeMap::new(),
            last_pos: None,
            last_signal: None,
            recent_positions: VecDeque::new(),
            ticks_since_signal_up: 0
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
                    last_seen_tick: state.tick
                },
            );
        }

        // add signal from this field to BTreeMap of all known tiles with a signal
        self.gem_signal_map.insert(state.bot, state.signal_level);
        
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
            if let Some(pos) = self.get_best_gem(state) {
                if let Some(gem) = self.known_gems.get(&pos) {
                    // eprintln!("{:?}, {}", self.current_target, state.tick);
                    self.current_target = Some(GemMemory {
                        position: pos,
                        ttl: gem.ttl,
                        last_seen_tick: state.tick
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

    pub fn get_best_gem(&self, state: &GameState) -> Option<[i32; 2]> {
        if self.known_gems.is_empty() {
            return None;
        }

        let mut best: Option<[i32; 2]> = None;
        let mut best_dist: i32 = i32::MAX;

        for (&gem_pos, gem_mem) in self.known_gems.iter() {
            let dist = match get_dist(state.bot, gem_pos, self) {
                Some(d) => d,
                None => continue, // unreachable gem
            };

            let age = state.tick - gem_mem.last_seen_tick; 
            let ttl_now = gem_mem.ttl - age as i32; 
            if ttl_now <= 0 { continue } // wenn gem despawnt ist

            if ttl_now <= dist { // if gem despawns before possible arrival if dist >= ttl_now { continue; }
                continue
            }

            if dist < best_dist {
                best = Some(gem_pos);
                best_dist = dist;
            } else if dist == best_dist {
                if let Some(curr) = best {
                    if let Some(closer) = get_closer_pos(state.bot, curr, gem_pos, self) {
                        best = Some(closer);
                    }
                }
            }
        }

        best
    }

}

pub fn get_closer_pos(
    goal: [i32; 2],
    a: [i32; 2],
    b: [i32; 2],
    memory: &BotMemory,
) -> Option<[i32; 2]> {
    let dist_a = get_dist(a, goal, memory);
    let dist_b = get_dist(b, goal, memory);

    match (dist_a, dist_b) {
        (Some(da), Some(db)) => {
            if da <= db { Some(a) } else { Some(b) }
        }
        (Some(_), None) => Some(a),
        (None, Some(_)) => Some(b),
        (None, None) => {
            eprintln!("both positions cant be reached");
            None
        }
    }
}