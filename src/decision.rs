use std::{
    collections::HashSet,
    i32,
};

use crate::signal::move_to_signal;
use crate::{memory::BotMemory, states::GameState};
use crate::explore::*;

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
        highlights.add_point(t, colors::RED);
        memory.signal_target = None;
        t
    }  
    else if state.signal_level > 0.0 {
        move_to_signal(memory, state)
    }
    else if let Some(pos) = memory.exploration_target { // gibt es ein exploration target
        highlights.add_point(pos, colors::GREEN);
        pos
    } 
    else if let Some(new_target) = get_exploration_target(state, memory) { // if no exploration target, pick new target
        memory.exploration_target = Some(new_target);
        highlights.add_point(new_target, colors::GREEN);
        new_target
    } 
    else { // if picking new target also failed, revert to movement fallback
        eprintln!("Kein Exploration Target gefunden - 404 :/");
        return movement_fallback(memory, state);
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
                colors::RED
            } else {
                colors::GREEN
            };
            highlights.add_path(path, path_color);
            

            return match (dx, dy) {
                (1, 0) => Direction::Right, // X+1 = rechts
                (-1, 0) => Direction::Left, // X-1 = links
                (0, 1) => Direction::Down, // Y+1 = unten
                (0, -1) => Direction::Up, // Y-1 = oben
                _ => {
                    eprintln!("UNEXPECTED BEHAVIOR: Unbekannte Bewegung");
                    return movement_fallback(&memory, state);
                }
            };
        } else {
            eprintln!("UNEXPECTED BEHAVIOR: Ziel wurde erreicht?");
            // Ziel wurde erreicht
            return movement_fallback(&memory, state);
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
    Direction::Wait
}

fn movement_fallback(memory: &BotMemory, state: &GameState) -> Direction {
    eprintln!("UNEXPECTED BEHAVIOR: FALLBACK MOVEMENT");
    // TODO: Das throwt jedes mal mein got
    let width = memory.config.width;
    let height = memory.config.height;
    let center = [width / 2, height / 2];
    let current = state.bot;
    
    // Richtung zur Mitte
    let dx = center[0] - current[0];
    let dy = center[1] - current[1];
    
    // Priorisierte Richtungen basierend auf Distanz zur Mitte
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
    
    // Finde erste gültige Richtung aus den priorisierten
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
    
    Direction::Wait // Wenn alle Richtungen blockiert sind
}