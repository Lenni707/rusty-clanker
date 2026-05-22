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

pub fn try_place_entrance(state: &GameState, memory: &BotMemory, id: usize) -> Option<Action> {
    let pos = state.bot;
    let candidates = [
        ([pos[0] + 1, pos[1]], "E"), // Rechts (X+1)
        ([pos[0] - 1, pos[1]], "W"), // Links (X-1)
        ([pos[0], pos[1] + 1], "S"), // Unten (Y+1)
        ([pos[0], pos[1] - 1], "N"), // Oben (Y-1)
    ];

    for (wall_pos, dir) in candidates {
        if memory.known_walls.contains(&wall_pos) 
        && !state.portal_stubs.contains(&wall_pos) { // TODO: Das noch mit den Sachen aus memory machen
            return Some(Action::PlacePortal { id, dir });
        }

        // TODO: Löchen das hier
        if state.portals.contains(&wall_pos) {
            if dir == "N" {
                return Some(Action::Move(Direction::Up));
            }
            if dir == "E" {
                return Some(Action::Move(Direction::Right));
            }
            if dir == "W" {
                return Some(Action::Move(Direction::Left));
            }
            if dir == "S" {
                return Some(Action::Move(Direction::Down));
            }
        }
    }
    None
}