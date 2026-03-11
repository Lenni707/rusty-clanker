use std::mem;
use std::sync::Arc;
use std::{
    collections::HashSet,
    i32,
};

use crate::signals::SignalSolver;
use crate::{memory::BotMemory, states::GameState};
use crate::{explore::*, signals};

use crate::astar::astar;
use crate::helper_func::*;

use crate::highlights::{Highlighter, colors};

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

// =====================================================
// Entscheidungsfunktion für den Bot
// =====================================================

pub fn decide(
    state: &GameState,
    memory: &mut BotMemory,
    highlights: &mut Highlighter
) -> Direction {
    let target = if let Some(t) = memory.get_target() { // Gibt es einen target gem
        // memory.signal_solver.update_measurenments(t, memory.config.signal_radius); // hahah ich weiß nicht warum aber das hier scheint besser zu sein
        memory.signal_solver.clear_measuremnts();
        highlights.add_point(t, colors::RED);
        memory.signal.signal_target = None;
        memory.signal.ticks_since_signal_up = 0;
        t
    }
    // else if let Some(t) = memory.get_signal_target(state.bot) {
    else if let Some (t) = memory.signal_solver.get_test_target(&memory.known_walls, &state.floor, &state.channels) {
        highlights.add_point(t, colors::PURPLE);
        t
    }
    else if let Some(pos) = memory.signal_solver.get_target(&memory.known_walls, &state.floor) { // TODO: Warum lebt SignalSovler in Memory??
        // TODO: DAS ALLES IN DIESEM ELSE BLOCK IST GRAUENHAFTTT!!!
        if astar(state.bot, pos, &memory.known_walls, memory.config.width, memory.config.height).is_none() {
            // Target not rachable
            eprintln!("TARGET NOT REACHABLE");
            return movement_fallback(memory, state, Some(pos)); // changes the target until it is reachable
        }
        highlights.add_point(pos, colors::PURPLE);
        // Check ob das Target Konstant ist:
        memory.last_signal_targets.push(pos);
        if memory.last_signal_targets.len() > 5 {
            memory.last_signal_targets.remove(0);
        }
        if memory.last_signal_targets.len() == 2 {
            let all_same = memory.last_signal_targets.iter().all(|&t| t == pos);
            if all_same {
                memory.signal_target = Some(pos);
            }
        }
        pos
    }
    else if let Some(pos) = memory.exploration_target { // gibt es ein exploration target
        highlights.add_point(pos, colors::GREEN);
        // if should_abort_exploration(&state, pos, &memory, memory.config.vis_radius) {
        //     memory.exploration_target = None;
        //     return Direction::Wait; // TODO: Hier noch einbauen, das direkt das nächste Ziel verwendet wird
        // }
        pos
    } 
    else if let Some(new_target) = get_exploration_target(state, memory) { // if no exploration target, pick new target
        memory.exploration_target = Some(new_target);
        if astar(state.bot, new_target, &memory.known_walls, memory.config.width, memory.config.height).is_none() {
            // Target not rachable
            eprintln!("TARGET NOT REACHABLE");
            return movement_fallback(memory, state, Some(new_target)); // changes the target until it is reachable
        }
        highlights.add_point(new_target, colors::GREEN);
        new_target
    } 
    else { // if picking new target also failed, revert to movement fallback
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
            } else if memory.signal_target.is_some() {
                colors::PURPLE // path bei signal
            } else {
                colors::GREEN // path bei explore
            };
            highlights.add_path(path, path_color);
            

            return match (dx, dy) {
                (1, 0) => Direction::Right, // X+1 = rechts
                (-1, 0) => Direction::Left, // X-1 = links
                (0, 1) => Direction::Down, // Y+1 = unten
                (0, -1) => Direction::Up, // Y-1 = oben
                _ => {
                    eprintln!("UNEXPECTED BEHAVIOR: Unbekannte Bewegung");
                    return movement_fallback(&memory, state, None);
                }
            };
        } else {
            eprintln!("UNEXPECTED BEHAVIOR: Ziel wurde erreicht?");
            memory.signal.signal_target = None; 
            // Ziel wurde erreicht
            return movement_fallback(&memory, state, None);
        }
    }

    memory.exploration_target = None;
    memory.signal.signal_target = None; 
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
    Direction::Wait
}

fn movement_fallback(memory: &BotMemory, state: &GameState, original_target: Option<Pos>) -> Direction {
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
                if let Some(path) = astar(current, candidate, &memory.known_walls, width, height) {
                    eprintln!("reachable {:?} (is offeseted {})", candidate, radius);
                    
                    if path.len() > 1 {
                        let next_step = path[1];
                        let dx = next_step[0] - current[0];
                        let dy = next_step[1] - current[1];
                        
                        return match (dx, dy) {
                            (1, 0) => Direction::Right,
                            (-1, 0) => Direction::Left,
                            (0, 1) => Direction::Down,
                            (0, -1) => Direction::Up,
                            _ => Direction::Wait,
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
            return dir;
        }
    }
    
    Direction::Wait
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