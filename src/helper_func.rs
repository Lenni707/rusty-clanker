use std::collections::{HashSet};

use crate::astar::astar;
use crate::memory::{BotMemory};
use crate::portals::Portal;

pub type Pos = [i32; 2]; // dachte iwie das kann tuff sein

// HILFSFUNKTIONEN
// 1. Manhatten Distance (heuristic)
// - Schätzt die Distanz zwischen 2 Punkten
pub fn heuristic(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}

pub fn heuristic_with_portals(a: Pos, b: Pos, portals: &Vec<Portal>) -> i32  { 
    let mut best = heuristic(a, b);

    for portal in portals {
        if !portal.is_complete() {
            continue;
        }

        let via_portal = heuristic(a, portal.entry.unwrap()) + heuristic(portal.exit_from.unwrap(), b);

        if via_portal < best {
            best = via_portal;
        }
    }

    best
}

pub fn euclidean(a: [i32; 2], b: [i32; 2]) -> f32 {
    let dx = (a[0] - b[0]) as f32;
    let dy = (a[1] - b[1]) as f32;
    (dx * dx + dy * dy).sqrt()
}

pub fn get_dist(
    a: [i32; 2],
    b: [i32; 2],
    known_walls: &HashSet<Pos>,
    width: i32,
    height: i32,
    portals: &Vec<Portal>,
) -> Option<i32> {
    let path = astar(a, b, &known_walls, portals, width, height);

    let dist = match path {
        Some(path) => Some(path.len() as i32 - 1),
        None => {
            eprintln!("theres no viable path between the points, so dist cant be measured");
            None
        }
    };

    dist
}

pub fn get_neighbours(pos: Pos) -> [Pos; 4] {
    let [x, y] = pos;

    [
        [pos[0] - 1, pos[1]],
        [pos[0] + 1, pos[1]], 
        [pos[0], pos[1]  - 1], 
        [pos[0], pos[1]  + 1]
    ]
}

pub fn is_in_bounds(pos: [i32; 2], width: i32, height: i32) -> bool {
    pos[0] >= 0 && pos[0] < width && pos[1] >= 0 && pos[1] < height
}

pub fn is_free(memory: &BotMemory, p: Pos) -> bool {
    is_in_bounds(p, memory.config.width, memory.config.height) && !memory.known_walls.contains(&p)
}

pub fn clamp_pos(p: Pos, w: i32, h: i32) -> Pos { // make sure pos is within maprange (pos < width and height)
    [p[0].clamp(0, w - 1), p[1].clamp(0, h - 1)]
}