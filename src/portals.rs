use crate::{astar::astar, decision::Action, helper_func::*, memory::{self, BotMemory}, states::GameState};
use crate::decision::Direction;
pub struct Portal {
    pub id: usize,
    pub entry: Option<Pos>,
    pub entry_from: Option<Pos>,
    pub exit: Option<Pos>,
    pub exit_from: Option<Pos>,
}

impl Portal {
    pub fn is_complete(&self) -> bool {
        self.entry.is_some() && self.exit.is_some()
    }
}

pub enum PortalAction {
    Place { portal_id: usize, direction: &'static str },
    None,
}

pub fn try_place_portal(state: &GameState, memory: &BotMemory, id: usize) -> Option<Action> {
    let pos = state.bot;
    let candidates = [
        ([pos[0] + 1, pos[1]], "E"), // Rechts (X+1)
        ([pos[0] - 1, pos[1]], "W"), // Links (X-1)
        ([pos[0], pos[1] + 1], "S"), // Unten (Y+1)
        ([pos[0], pos[1] - 1], "N"), // Oben (Y-1)
    ];

    for (wall_pos, dir) in candidates {
        if memory.known_walls.contains(&wall_pos) && !state.portal_stubs.contains(&wall_pos) && !state.portals.contains(&wall_pos) {
            return Some(Action::PlacePortal { id, dir });
        }
        continue;
    }
    None
}

fn get_portal_potential_at(state: &GameState, memory: &BotMemory)  { //  -> f32
    let score = 0.2; // base score

    // factor in distance to set point
    let width = memory.config.width;
    let height = memory.config.height;

    let corner_top_left = [width * (1/4), (height * (3/4))];
    let corner_top_right = [width * (3/4), (height * (3/4))];
    let corner_bottom_left = [width * (1/4), (height * (1/4))];
    let corner_bottom_right = [width * (3/4), (height * (1/4))];


}
// =========
// Bewertet Portalpaare anhand von der Distanz, die sie überspringen
// =========
// Problem: Erst wirklich gut, wenn die Map gut erkannt ist.
fn score_portal_pair(
    start_pos: Pos,
    end_pos: Pos,
    memory: &BotMemory,
) -> f32 {
    let real_path = astar(start_pos, end_pos, &memory.known_walls, &memory.portals, memory.config.width, memory.config.height);
    let real_dist = real_path.map(|p| p.len()).unwrap_or(676767) as f32;

    // Luftlinie: wenn es geringe Luftlinie aber sonst große Distanz ist
    let air_line = heuristic(start_pos, end_pos) as f32;

    let score = real_dist - air_line;

    score
}
// =========
// Findet gute Ausgänge für Portale mit bisher nur einem Eingang
// =========
fn find_best_portal_exit(entrance_pos: Pos, memory: &BotMemory) -> Option<Pos> {
    let mut best_score = 0.0;
    let mut best_pos = None;

    for wall in &memory.known_vis_walls {
        // Nachbarn, also die Felder, wo dann gelandet wird. Die Floor neighbors
        for neighbor in get_floor_neighbors(*wall, memory) {
            let score = score_portal_pair(entrance_pos, neighbor, memory);

            if score > best_score {
                best_pos = Some(*wall);
                best_score = score;
            }
        }
    }
    best_pos
}

fn get_floor_neighbors(pos: Pos, memory: &BotMemory) -> Vec<Pos> {
    let candidates = [
        ([pos[0] + 1, pos[1]], "E"), // Rechts (X+1)
        ([pos[0] - 1, pos[1]], "W"), // Links (X-1)
        ([pos[0], pos[1] + 1], "S"), // Unten (Y+1)
        ([pos[0], pos[1] - 1], "N"), // Oben (Y-1)
    ];
    let mut floor_neighbors = Vec::new();
    for (candidate,_) in candidates {
        if memory.known_floors.contains(&candidate) {
            floor_neighbors.push(candidate);
        }
    }
    floor_neighbors
}
// IDEE: Erstmal am anfang irgendwo ein Portal platzieren, dann einen guten Exit finden...
