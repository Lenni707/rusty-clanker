use std::{
    collections::HashSet,
    i32,
};

use crate::{memory::BotMemory, states::GameState};
use crate::explore::*;

use crate::astar::astar;
use crate::helper_func::*;

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
    width: i32,
    height: i32,
) -> (Direction, Option<Vec<[i32; 2]>>) {
    // Gibt es einen target gem?
    let target_gem = match memory.get_target() {
        // müsst eeigenltich target heißen
        Some(t) => t,
        None => {
            // Kein Target Gem-> Exploreation
            match memory.exploration_target {
                Some(explored_pos) => {
                    //if should_abort_exploration(state, explored_pos, memory, 10) {
                    //    memory.exploration_target = None;
                    //    // Auf nächsten frame warten, TODO: hier immer das nächste Ziel aussuchen
                    //    return (Direction::Wait, None);
                    //}
                    explored_pos},
                None => {
                    // Kein Aktuelles Target: Neues Target finden
                    match get_exploration_target(state, memory, width, height) {
                        Some(new_target) => {
                            memory.exploration_target = Some(new_target);
                            new_target
                        }
                        None => {
                            eprintln!("Kein Exploration Target gefunden - 404 :/");
                            return (movement_fallback(&memory, &state), None);
                            // return Direction::Wait;
                        }
                    }
                }
            }
        }
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
    let path = astar(state.bot, target_gem, &walls, width, height);

    // Wenn wir einen Pfad gefunden haben:
    if let Some(path) = path {
        // path enthält: [Start, Schritt1, Schritt2, ..., Ziel]
        if path.len() > 1 {
            let next_step = path[1];

            let dx = next_step[0] - state.bot[0];
            let dy = next_step[1] - state.bot[1];

            return match (dx, dy) {
                (1, 0) => (Direction::Right, Some(path)), // X+1 = rechts
                (-1, 0) => (Direction::Left, Some(path)), // X-1 = links
                (0, 1) => (Direction::Down, Some(path)),  // Y+1 = unten
                (0, -1) => (Direction::Up, Some(path)),   // Y-1 = oben
                _ => {
                    eprintln!("UNEXPECTED BEHAVIOR: Unbekannte Bewegung");
                    return (movement_fallback(&memory, state), None);
                }
            };
        } else {
            eprintln!("UNEXPECTED BEHAVIOR: Ziel wurde erreicht?");
            // Ziel wurde erreicht
            return (movement_fallback(&memory, state), None);
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
    (Direction::Wait, None)
}

fn movement_fallback(memory: &BotMemory, state: &GameState) -> Direction {
    eprintln!("UNEXPECTED BEHAVIOR: FALLBACK MOVEMENT");
    // TODO: Das throwt jedes mal mein got
    let config = state.config.as_ref().unwrap();
    let center = [config.width / 2, config.height / 2];
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
        
        if is_in_bounds(next_pos, config.width, config.height) 
           && !memory.known_walls.contains(&next_pos) {
            return dir;
        }
    }
    
    Direction::Wait // Wenn alle Richtungen blockiert sind
}