use crate::{decision::Action, helper_func::*, memory::{self, BotMemory}, states::GameState};
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