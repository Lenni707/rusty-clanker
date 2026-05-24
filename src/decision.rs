use std::{
    collections::HashSet,
    i32,
};

use crate::memory::extract_signal_target_pos;
use crate::portals::{Portal, PortalAction};
use crate::portals::*;
use crate::signals::SignalSolver;
use crate::{memory::BotMemory, states::GameState};
use crate::{explore::*, signals};


use crate::astar::astar;
use crate::helper_func::*;

use crate::highlights::{Highlighter, colors};

pub enum Action {
    Move(Direction),
    PlacePortal { id: usize, dir: &'static str },
    Wait,
}


impl Action {
    pub fn as_command(&self) -> String {
        match self {
            Action::Move(dir) => dir.as_geo().to_string(),
            Action::PlacePortal { id, dir } => format!("P{}{}", id, dir),
            Action::Wait => "WAIT".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
    Wait,
}

impl Direction {
    pub fn as_geo(&self) -> &'static str {
        match self {
            Direction::Down => "S",
            Direction::Up => "N",
            Direction::Left => "W",
            Direction::Right => "E",
            _ => "WAIT",
        }
    }
}

fn step_along_path(path: &[[i32; 2]], bot: Pos, memory: &BotMemory, state: &GameState) -> Action {
    let next = path[1];
    match (next[0] - bot[0], next[1] - bot[1]) {
        (1, 0)  => Action::Move(Direction::Right),
        (-1, 0) => Action::Move(Direction::Left),
        (0, 1)  => Action::Move(Direction::Down),
        (0, -1) => Action::Move(Direction::Up),
        _       => movement_fallback(memory, state, None),
    }
}
fn is_adjacent(a: Pos, b: Pos) -> bool {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs() == 1
}

fn get_direction_to(from: Pos, to: Pos) -> Option<&'static str> {
    match (to[0] - from[0], to[1] - from[1]) {
        (1, 0)  => Some("E"),
        (-1, 0) => Some("W"),
        (0, 1)  => Some("S"),
        (0, -1) => Some("N"),
        _       => None,
    }
}

// =====================================================
// Entscheidungsfunktion für den Bot
// =====================================================

pub fn decide(
    state: &GameState,
    memory: &mut BotMemory,
    highlights: &mut Highlighter
) -> Action {
    let gem_target = memory.get_target();
    let signal_target = extract_signal_target_pos(memory.signal_target);

    // --- Portal Platzieren (nur wenn kein Gem/Signal drängt) ---
    if gem_target.is_none() && signal_target.is_none() {

        // Eingang platzieren
        if memory.portal_entry_placed.is_none() {
            if let Some(wall_target) = memory.portal_entry_target {
                if is_adjacent(state.bot, wall_target) {
                    if let Some(dir_str) = get_direction_to(state.bot, wall_target) {
                        memory.portal_entry_placed = Some(wall_target);
                        memory.portal_entry_target = None;
                        return Action::PlacePortal { id: 1, dir: dir_str };
                    }
                } else {
                    highlights.add_point(wall_target, colors::YELLOW);
                    if let Some(path) = astar(state.bot, wall_target, &memory.known_walls, &memory.portals, memory.config.width, memory.config.height) {
                        if path.len() > 1 {
                            return step_along_path(&path, state.bot, memory, state);
                        }
                    }
                    // Nicht erreichbar → neu suchen nächsten Tick
                    memory.portal_entry_target = None;
                }
            }
        }

        // Ausgang platzieren
        if memory.portal_entry_placed.is_some() && memory.portal_exit_placed.is_none() {
            if let Some(wall_target) = memory.portal_exit_target {
                if is_adjacent(state.bot, wall_target) {
                    if let Some(dir_str) = get_direction_to(state.bot, wall_target) {
                        memory.portal_exit_placed = Some(wall_target);
                        memory.portal_exit_target = None;
                        return Action::PlacePortal { id: 1, dir: dir_str };
                    }
                } else {
                    highlights.add_point(wall_target, colors::YELLOW);
                    if let Some(path) = astar(state.bot, wall_target, &memory.known_walls, &memory.portals, memory.config.width, memory.config.height) {
                        if path.len() > 1 {
                            return step_along_path(&path, state.bot, memory, state);
                        }
                    }
                    // Nicht erreichbar → neu suchen nächsten Tick
                    memory.portal_exit_target = None;
                }
            }
        }
    }

    // --- Normaler Target-Flow ---
    let target = if let Some(t) = gem_target {
        highlights.add_point(t, colors::RED);
        t
    } else if let Some(t) = signal_target {
        highlights.add_point(t, colors::CYAN);
        t
    } else if let Some(pos) = memory.exploration_target {
        highlights.add_point(pos, colors::GREEN);
        pos
    } else if let Some(new_target) = get_exploration_target(state, memory) {
        memory.exploration_target = Some(new_target);
        if astar(state.bot, new_target, &memory.known_walls, &memory.portals, memory.config.width, memory.config.height).is_none() {
            eprintln!("TARGET NOT REACHABLE");
            return movement_fallback(memory, state, Some(new_target));
        }
        highlights.add_point(new_target, colors::GREEN);
        new_target
    } else {
        eprintln!("Kein Exploration Target gefunden - 404 :/");
        return movement_fallback(memory, state, None);
    };

    let mut walls: HashSet<[i32; 2]> = memory.known_walls.clone();

    // Den Gegernerischen Bot in die Walls hinzu fügen
    if let Some(bots) = &state.visible_bots {
        for bot in bots {
            let dist_bots = heuristic(state.bot, bot.position);
            if dist_bots < 3 { 
                walls.insert(bot.position);
            }
        }
    }

    // Finde den Weg mit A*
    let path = astar(
        state.bot, 
        target, 
        &walls, 
        &memory.portals,
        memory.config.width, 
        memory.config.height
    );

    // Wenn wir einen Pfad gefunden haben move through path:
    if let Some(path) = path {
        // path enthält: [Start, Schritt1, Schritt2, ..., Ziel]
        if path.len() > 1 {
            let next_step = path[1];

            let dx = next_step[0] - state.bot[0];
            let dy = next_step[1] - state.bot[1];

            // Pfad highlighten
            let path_color = if memory.get_target().is_some() {
                colors::RED // path bei gem
            } else if memory.exploration_target.is_none() {
                colors::PURPLE // path bei explore
            } else {
                colors::GREEN // path bei signal
            };
            highlights.add_path(path, path_color);
            

            return match (dx, dy) {
                (1, 0) => Action::Move(Direction::Right), // X+1 = rechts
                (-1, 0) => Action::Move(Direction::Left), // X-1 = links
                (0, 1) => Action::Move(Direction::Down), // Y+1 = unten
                (0, -1) => Action::Move(Direction::Up), // Y-1 = oben
                _ => {
                    eprintln!("UNEXPECTED BEHAVIOR: Unbekannte Bewegung");
                    return movement_fallback(&memory, state, None);
                }
            };
        } else {
            eprintln!("UNEXPECTED BEHAVIOR: Ziel wurde erreicht?");
            // Ziel wurde erreicht
            return movement_fallback(&memory, state, None);
        }
    }

    memory.exploration_target = None;
    eprintln!("Es wurde kein Pfad gefunden :/");
    // Check, um unterschiede zwischen den sichtbaren gems und den gespeicherten Gems zu finden
    for gem in &state.visible_gems {
        let mut iter_check = false;
        for supposed_gem in &memory.known_gems {
            if supposed_gem.0 == &gem.position {
                iter_check = true;
            }
        }
        if !iter_check {
            eprintln!("DISCRAPENCY: {:?}", gem.position)
        }
    }
    Action::Wait
}

fn movement_fallback(memory: &BotMemory, state: &GameState, original_target: Option<Pos>) -> Action {
    eprintln!("UNEXPECTED BEHAVIOR: FALLBACK MOVEMENT");
    
    let width = memory.config.width;
    let height = memory.config.height;
    let current = state.bot;
    
    // If we have an original target, try to find a reachable position near it
    if let Some(target) = original_target {
        eprintln!("trying to find new reachable pos {:?}", target);
        
        // search in expanding radius around the original target
        for radius in 1..=10 {
            let offsets = get_positions_at_radius(radius);
            
            for offset in offsets {
                let candidate = [target[0] + offset[0], target[1] + offset[1]];
                
                // Check if candidate is valid
                if candidate[0] < 0 || candidate[0] >= width || 
                   candidate[1] < 0 || candidate[1] >= height {
                    continue;
                }
                
                if memory.known_walls.contains(&candidate) {
                    continue;
                }
                
                // path to this canidate
                if let Some(path) = astar(current, candidate, &memory.known_walls, &memory.portals, width, height) {
                    eprintln!("reachable {:?} (is offeseted {})", candidate, radius);
                    
                    if path.len() > 1 {
                        let next_step = path[1];
                        let dx = next_step[0] - current[0];
                        let dy = next_step[1] - current[1];
                        
                        return match (dx, dy) {
                            (1, 0) => Action::Move(Direction::Right), // X+1 = rechts
                            (-1, 0) => Action::Move(Direction::Left), // X-1 = links
                            (0, 1) => Action::Move(Direction::Down), // Y+1 = unten
                            (0, -1) => Action::Move(Direction::Up), // Y-1 = oben
                            _ => Action::Wait
                        };
                    }
                }
            }
        }
        
        eprintln!("no reachable near target");
    }
    
    // Fallback: move towards center
    let center = [width / 2, height / 2];
    let dx = center[0] - current[0];
    let dy = center[1] - current[1];
    
    let preferred_directions = if dx.abs() > dy.abs() {
        if dx > 0 {
            vec![Direction::Right, Direction::Down, Direction::Up, Direction::Left]
        } else {
            vec![Direction::Left, Direction::Down, Direction::Up, Direction::Right]
        }
    } else {
        if dy > 0 {
            vec![Direction::Down, Direction::Right, Direction::Left, Direction::Up]
        } else {
            vec![Direction::Up, Direction::Right, Direction::Left, Direction::Down]
        }
    };
    
    for dir in preferred_directions {
        let next_pos = match dir {
            Direction::Right => [current[0] + 1, current[1]],
            Direction::Left => [current[0] - 1, current[1]],
            Direction::Down => [current[0], current[1] + 1],
            Direction::Up => [current[0], current[1] - 1],
            Direction::Wait => current,
        };
        
        if is_in_bounds(next_pos, width, height) 
           && !memory.known_walls.contains(&next_pos) {
            return Action::Move(dir);
        }
    }
    
    Action::Wait
}

fn get_positions_at_radius(radius: i32) -> Vec<[i32; 2]> {
    let mut positions = Vec::new();
    
    for dx in -radius..=radius {
        let remaining = radius - dx.abs();
        if remaining == 0 {
            positions.push([dx, 0]);
        } else {
            positions.push([dx, remaining]);
            positions.push([dx, -remaining]);
        }
    }
    
    positions
}