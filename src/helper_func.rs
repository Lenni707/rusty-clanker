use crate::astar::astar;
use crate::memory::BotMemory;

// HILFSFUNKTIONEN
// 1. Manhatten Distance (heuristic)
// - Schätzt die Distanz zwischen 2 Punkten
pub fn heuristic(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}

pub fn is_in_bounds(pos: [i32; 2], width: i32, height: i32) -> bool {
    pos[0] >= 0 && pos[0] < width && pos[1] >= 0 && pos[1] < height
}

pub fn get_dist(
    a: [i32; 2],
    b: [i32; 2],
    memory: &BotMemory,
) -> Option<i32> {
    let path = astar(a, b, &memory.known_walls, memory.width, memory.height);

    let dist = match path {
        Some(path) => Some(path.len() as i32 - 1),
        None => {
            eprintln!("theres no viable path between the points, so dist cant be measured");
            None
        }
    };

    dist
}