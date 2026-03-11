use std::collections::{BTreeMap, HashSet};
use std::collections::VecDeque; use crate::astar::{self, astar};
use crate::explore::has_line_of_sight;
use crate::highlights::{self, Highlighter, colors};
// iwie effiezient oder so ka
use crate::states::{GameState, Config};

use crate::helper_func::{get_dist, Pos};
use crate::signals::{self, SignalSolver};

pub struct BotMemory {
    pub config: Config,

    pub current_target: Option<GemMemory>,
    pub known_gems: BTreeMap<[i32; 2], GemMemory>,
    pub known_positions: BTreeMap<[i32; 2], PositionMemory>, // ich weiß wirklich nicht wieso positions, das sind eigentlich die floors
    pub known_walls: HashSet<[i32; 2]>,
    pub known_floors: HashSet<[i32; 2]>, // Das ist nur fürs einfacheren arbeiten, anstatt mit known_positions
    pub reachable_positions: HashSet<Pos>,
    pub exploration_target: Option<[i32; 2]>,
    pub gem_spawn_heatmap: BTreeMap<[i32; 2], u32>, // pub last_steps

    pub last_pos: Option<Pos>,
    pub recent_positions: VecDeque<Pos>,

    pub signal_solver: SignalSolver,

    pub signal: SignalStats,
    pub signal_target: Option<Pos>,
    pub last_signal_targets: Vec<Pos>,
    pub ticks_since_last_signal: i32,
}

pub struct SignalStats {
    pub signal_target: Option<Pos>,
    pub gem_signal_map: BTreeMap<[i32; 2], f32>,
    pub last_signal: Option<f32>,
    pub ticks_since_signal_up: i32,
    pub best_signal: f32,
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

impl SignalStats {
    fn new() -> Self {
        SignalStats {
            signal_target: None,
            gem_signal_map: BTreeMap::new(),
            last_signal: None,
            ticks_since_signal_up: 0,
            best_signal: 0.0,
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
            known_floors: HashSet::new(),
            reachable_positions: HashSet::new(),
            exploration_target: None,
            gem_spawn_heatmap: BTreeMap::new(),

            last_pos: None,
            recent_positions: VecDeque::new(),

            signal_solver: SignalSolver::new(49, 29, 5.0, [5, 6], 0.03, HashSet::new()), // Dummy Werte, werden beim ersten Tick überschrieben, sollten aber zur Not funktionieren

            signal: SignalStats::new(),
            signal_target: None,
            last_signal_targets: Vec::new(),
            ticks_since_last_signal: 0,
        }
    }
    pub fn get_reachable_positions(&self, state: &GameState) -> HashSet<Pos> {
        let mut reachable = HashSet::new();
        let mut stack = vec![state.bot];
        
        while let Some(current) = stack.pop() {
            if reachable.contains(&current) {
                continue;
            }
            
            // wenn es eine known wall ist skippen
            if self.known_walls.contains(&current) {
                continue;
            }
            
            // wird dann reachable
            reachable.insert(current);
            
            // versuchen alles nachbarn des feldes zu flodden
            let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
            for dir in &directions {
                let next = [current[0] + dir[0], current[1] + dir[1]];
                
                // ob es noch in bounds ist
                if next[0] < 0 || next[0] >= self.config.width || 
                   next[1] < 0 || next[1] >= self.config.height {
                    continue;
                }
                
                // nicht flodden wenn wand
                if self.known_walls.contains(&next) {
                    continue;
                }
                
                // wenn noch nicht visited zu staxk pushen (eig unnötig weil oben schon gecjhecjkt wird)
                if !reachable.contains(&next) {
                    stack.push(next);
                }
            }
        }
        
        reachable
    }

    pub fn get_wall_positions(&self, state: &GameState) -> Vec<Pos> {
        let reachable = self.get_reachable_positions(state);

        let mut walls = Vec::new();
        for x in 0..self.config.width {
            for y in 0..self.config.height {
                let p: Pos = [x, y];
                if !reachable.contains(&p) {
                    walls.push(p);
                }
            }
        }
        walls
    }
    // Update: Wurden neue gems gesehen? Läuft vor decide
    pub fn update(&mut self, state: &GameState, highlights: &mut Highlighter) {
        // Signal Solver Intitialisieren, sobald die config verfügbar ist
        if state.tick == 1 && self.config.width > 0 {
            self.signal_solver = SignalSolver::new(
                self.config.width, 
                self.config.height, 
                self.config.signal_radius,
                [0, 0], // mock value fuck borrow checker
                self.config.signal_noise,
                HashSet::new()
            );
        }

        self.signal_solver.bot_pos = state.bot;

        if state.signal_level > 0.0 {
            self.ticks_since_last_signal = 0;
            self.signal_solver.add_measurement(&state.channels, state.bot);
        } else {
            self.ticks_since_last_signal += 1;
        }
        
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
            self.known_floors.insert(*floor_pos); // goofy ik ik
        }
        // Alle sichtbaren Walls markieren
        for wall_pos in &state.wall {
            self.known_walls.insert(*wall_pos);
        }

        for pos in self.get_reachable_positions(state) {
            self.reachable_positions.insert(pos);
        }

        for pos in self.get_wall_positions(state) { // insert calculated hidden walls into known walls
            self.known_walls.insert(pos);
        }

        self.signal_solver.known_walls = self.known_walls.clone(); // ich hasse clone omg // für signal solver

        highlights.add_path_hashset(&self.known_walls, colors::LIGHT_GREY); // markiert alle known walls

        highlights.add_point([0, 0], colors::GREEN);

        // self.known_positions.retain(|_, &mut last_seen| state.tick - last_seen < 250);

        // Check für das current target
        let mut change_curr_target = false;
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
                change_curr_target = true;
            }
            // Auch wenn der andere Bot näher ist:
            if let Some(enemy_bots) = &state.visible_bots {
                for bot in enemy_bots {
                    // Wenn gegnerischer Bot gem sehen kann
                    if has_line_of_sight(bot.position, target.position, &self.known_walls, self.config.vis_radius) {
                        if let Some(enemy_path) = astar(bot.position, target.position, &self.known_walls, self.config.width, self.config.height) {
                            if let  Some(own_path) = astar(state.bot, target.position, &self.known_walls, self.config.width, self.config.height) {
                                if enemy_path.len() + 2 < own_path.len() {
                                    change_curr_target = true;
                                    self.known_gems.remove(&target.position);
                                    //self
                                }
                            }
                        }
                    }
                }
            }
        }
        if change_curr_target {
            self.current_target = None;
        }
        // Signal Target Check; Wurde es erreicht?
        let mut change_sig_target = false;
        if let Some(target) = &self.signal_target {
            if self.ticks_since_last_signal > 5 {
                change_sig_target = true;
                eprint!("ÖLASDÖFDSLJFKÖLSDFJÖSJDFÖSLFJKÖSDFJ")
            }
            // Wenn es ein Target gibt: Dann nichtmehr zum Exploration Target zurück gehen
            self.exploration_target = None;
            // Checks für curr target
            let reached = &state.bot == target;
            let mut visible = false;
            if has_line_of_sight(state.bot, *target, &self.known_walls, self.config.vis_radius) {
                visible = true;
            }
            if reached || (!visible && state.floor.contains(&target)) {
                // TODO: Das ist komisch hier, aber wenn es ein ODER ist dann geht es kaputt
                change_sig_target = true;
            }
            // Auch wenn der andere Bot näher ist:
            if let Some(enemy_bots) = &state.visible_bots {
                for bot in enemy_bots {
                    // Wenn gegnerischer Bot gem sehen kann
                    if has_line_of_sight(bot.position, *target, &self.known_walls, self.config.vis_radius) {
                        if let Some(enemy_path) = astar(bot.position, *target, &self.known_walls, self.config.width, self.config.height) {
                            if let  Some(own_path) = astar(state.bot, *target, &self.known_walls, self.config.width, self.config.height) {
                                if enemy_path.len() + 2 < own_path.len() {
                                    change_sig_target = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if change_sig_target {
            self.signal_target = None;
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
        // Gem-Target hat Priorität
        if let Some(target) = &self.current_target {
            return Some(target.position);
        }
        
        None
    }
    pub fn get_signal_target(&mut self, bot_pos: Pos) -> Option<Pos> {
        if astar(bot_pos, self.signal_target?, &self.known_walls, self.config.width, self.config.height).is_none() {
            self.signal_target = None;
            self.last_signal_targets.clear();
            eprintln!("No way found, reseting signal_taret");
            return None;
        }
        match &self.signal_target {
            Some(t) => Some(*t),
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
            let dist = match get_dist(state.bot, gem_pos, &self.known_walls, self.config.width, self.config.height) {
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
    let dist_a = get_dist(a, goal, &memory.known_walls, memory.config.width, memory.config.height);
    let dist_b = get_dist(b, goal, &memory.known_walls, memory.config.width, memory.config.height);

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