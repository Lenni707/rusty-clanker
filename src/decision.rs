use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    i32,
};

use rand::Rng;

use crate::{memory::BotMemory, states::GameState};

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
// PATHFINDING
// - via dem A* algorithmus
// - verwenden wie hier, um den kürzesten Weg zwischen 2 Punkten mit dem geringsten Kost zu finden
// =====================================================

#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: [i32; 2], // Wo ist der zu erkundende Knoten / Punkt
    g_cost: i32,        // Wie viele Schritte vom Start bis dorthin?
    f_cost: i32,        // Geschätze Kosten via heuristic bis zum Ziel
}

// VERGLEICHEN VON STRECKEN
// Wir wollten der der Node, mit dem kleinsten f_cost, welcher als "größer" gilt
// Das wird dann in einem BinaryHeap gespeichert. Dieser sortiert alle Daten nach der Größe, bei und also das kleine zu erst
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost)) // mein Gott
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// HILFSFUNKTIONEN
// 1. Manhatten Distance (heuristic)
// - Schätzt die Distanz zwischen 2 Punkten
fn heuristic(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}
// 2. Gibt alle Nachbarn zurück, no shit right?
fn get_neighbors(pos: [i32; 2]) -> Vec<[i32; 2]> {
    vec![
        [pos[0] + 1, pos[1]], // Rechts (X+1)
        [pos[0] - 1, pos[1]], // Links (X-1)
        [pos[0], pos[1] + 1], // Unten (Y+1)
        [pos[0], pos[1] - 1], // Oben (Y-1)
    ]
}

fn is_in_bounds(pos: [i32; 2], width: i32, height: i32) -> bool {
    pos[0] >= 0 && pos[0] < width && pos[1] >= 0 && pos[1] < height
}

// =====================================================
// A* Pathfinding: Hauptalgorythmus
// =====================================================

fn astar(
    start: [i32; 2],
    goal: [i32; 2],
    walls: &HashSet<[i32; 2]>,
    width: i32,
    height: i32,
) -> Option<Vec<[i32; 2]>> {
    // OPENSET: Priority Queue zu dem nächsten Besuchbaren knoten
    let mut open_set = BinaryHeap::new();

    // Speichert jede Position, aus der wir gekommen sind.
    // Bsp: came_from[[3, 4]] = [2, 4] -> um zu 3, 4 zu kommen sind wir von 2, 4 aus gekommen
    let mut came_from: HashMap<[i32; 2], [i32; 2]> = HashMap::new();

    // Speichet die Kosten von Start zu jeder Position
    // g_score[[3, 4]] = 5 bedeutet, wir haben 5 Schritte gebraucht um 3, 4 zu erreichen
    let mut g_score: HashMap<[i32; 2], i32> = HashMap::new();

    // Startposition hat g_cost = 0, da wir da bereits sind
    g_score.insert(start, 0);

    // Startknoten in open_set hinzufügen
    open_set.push(Node {
        position: start,
        g_cost: 0,
        f_cost: heuristic(start, goal),
    });
    // Hauptschleife
    // - wenn es einen Knoten zum Untersuchen gibt:
    while let Some(current) = open_set.pop() {
        // Ist das Ziel bereits erreicht?
        if current.position == goal {
            //eprintln!("Ziel erreicht");
            // Den Pfand rekonstruieren
            let mut path = vec![current.position];
            let mut pos = current.position;

            // Folgen des "came_from" Links bis zum Start
            while let Some(&previous) = came_from.get(&pos) {
                path.push(previous);
                pos = previous;
            }

            // Pfad umdrehen, damit Start -> Ziel
            path.reverse();

            // eprintln!("Pfad gefunden mit {} Schritten", path.len());
            return Some(path);
        }

        // Ist das Ziel nicht erreicht:
        // NACHBARN UNTERSUCHEN

        for neighbor_pos in get_neighbors(current.position) {
            // Ist der Nachbar im Spielfeld, ist das überhaupt relevant hier?
            if !is_in_bounds(neighbor_pos, width, height) {
                continue;
            }
            // Ist der Nachbar eine Wand?
            if walls.contains(&neighbor_pos) {
                continue; // Wenn ja: Überspringen
            }

            // Berechnen der Kosten, von current bis zu neighbor
            let tentative_g_score = g_score.get(&current.position).unwrap_or(&i32::MAX) + 1; // Bei Fehler: Den Maximal möglichen Wert nehmen, +1 von curr zu neigh

            let neighbor_g_score = *g_score.get(&neighbor_pos).unwrap_or(&i32::MAX);

            // Haben wir einen besseren Weg gefunden?
            if tentative_g_score < neighbor_g_score {
                came_from.insert(neighbor_pos, current.position);
                g_score.insert(neighbor_pos, tentative_g_score);

                // Füge neighbor zur open_set hinzu (oder update ihn)
                let h = heuristic(neighbor_pos, goal);
                open_set.push(Node {
                    position: neighbor_pos,
                    g_cost: tentative_g_score,
                    f_cost: tentative_g_score + h, // g + h = f
                });
            }
        }
    }
    eprintln!("UNEXPECTED BEHAIVIOR: Keinen Weg zum Ziel gefunden");
    None
}

// =====================================================
// EXPLORATION von unbekannten Orten (spooky)
// =====================================================

fn get_exploration_target(state: &GameState, memory: &BotMemory) -> Option<[i32; 2]> {
    // Verwendet eine Liste aus kürzlich besuchten tiles, um neune zu unbekannt Tiles zu besuchen
    let curr_tick = state.tick;
    let curr_floor_count = state.floor.len();
    let mut candidates = Vec::new();

    
    let mut all_positions: HashSet<[i32; 2]> = HashSet::new();

    for &floor_pos in &state.floor {
        all_positions.insert(floor_pos);
    }
    for &floor_pos in memory.known_positions.keys() {
        all_positions.insert(floor_pos);
    }

    for &floor_pos in &all_positions {
        let seen = memory.known_positions.get(&floor_pos);

        let (interest_score, exploration_bonus) = match seen {
            None => (1000, 500),
            Some(&mem) => {
                let age = curr_tick - mem.last_seen_tick;
                // if age < 30 {
                //     continue; // Kürzlichst besucht, nicht hingehen
                // }
                let floor_diff = (curr_floor_count as i32 - mem.vis_floor_count as i32).abs();
                let exploration_bonus = floor_diff * 2;

                (age, exploration_bonus as u32)
            }
        };
        let distance = heuristic(state.bot, floor_pos);

        let spawn_bonus = memory.gem_spawn_heatmap.get(&floor_pos).unwrap_or(&0) * 10;
        // eprintln!("{spawn_bonus}");
        let score = interest_score as i32 + exploration_bonus as i32 + spawn_bonus as i32 - distance;

        candidates.push((floor_pos, score));
    }
    candidates.sort_by_key(|(_, score)| -score);

    candidates.first().map(|(pos, _)| *pos)
}

// =====================================================
// Entscheidungsfunktion für den Bot
// =====================================================

pub fn decide(state: &GameState, memory: &mut BotMemory, width: i32, height: i32) -> (Direction, Option<Vec<[i32; 2]>>) {
    // Gibt es einen target gem?
    let target_gem = match memory.get_target() {
        // müsst eeigenltich target heißen
        Some(t) => t,
        None => {
            // Kein Target Gem-> Exploreation
            match memory.exploration_target {
                Some(explored_pos) => explored_pos,
                None => {
                    // Kein Aktuelles Target: Neues Target finden
                    match get_exploration_target(state, memory) {
                        Some(new_target) => {
                            memory.exploration_target = Some(new_target);
                            eprintln!("Started exploring, {:?}", state.tick);
                            new_target
                        }
                        None => {
                            eprintln!("Kein Exploration Target gefunden - 404 :/");
                            return (move_random(), None);
                            // return Direction::Wait;
                        }
                    }
                }
            }
        }
    };

    let walls:&HashSet<[i32; 2]> = &memory.known_walls;

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
                (-1, 0) =>( Direction::Left, Some(path)), // X-1 = links
                (0, 1) => (Direction::Down, Some(path)),  // Y+1 = unten
                (0, -1) =>( Direction::Up, Some(path)),   // Y-1 = oben
                _ => {
                    eprintln!("UNEXPECTED BEHAVIOR: Unbekannte Bewegung");
                    return (move_random(), None);
                }
            };
        } else {
            eprintln!("UNEXPECTED BEHAVIOR: Ziel wurde erreicht?");
            // Ziel wurde erreicht
            return (move_random(), None);
        }
    }
    eprintln!("Es wurde kein Pfad gefunden :/");
    // Check, um unterschiede zwischen den sichtbaren gems und den gespeicherten Gems zu finden
    eprintln!("{:?}", memory.known_gems);
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

fn move_random() -> Direction {
    eprintln!("UNEXPECTED BEHAVIOR: FALLBACK TO RANDOM MOVMENT");
    let mut rng = rand::rng();
    let directions = [
        Direction::Down,
        Direction::Up,
        Direction::Left,
        Direction::Right,
    ];
    return directions[rng.random_range(0..directions.len())];
}

// pub fn decide_v1(state: &GameState, memory: &mut BotMemory) -> Direction {
//     let bot = state.bot;
//     let bot_pos = (bot[0], bot[1]);
//     memory.mark_visited((bot[0], bot[1]));

//     // Wenn ein Gems da ist: Hin gehen
//     if let Some(gem) = state.visible_gems.first() {
//         eprintln!("Visible gems: {}", state.visible_gems.len());
//         // first da sonst idx out of range, sofern kein GEM das ist
//         let gem_pos = gem.position;

//         return move_towards(&bot_pos, (gem_pos[0], gem_pos[1]), &state);
//     }
//     // Wenn keine Gems vorhanden: Eine unbekannte, neue zufällige Richtung probieren
//     let directions = [
//         Direction::Up,
//         Direction::Down,
//         Direction::Left,
//         Direction::Right,
//     ];
//     let mut rng = rand::rng();

//     for dir in directions.choose_multiple(&mut rng, directions.len()) {
//         let next_pos = match dir {
//             Direction::Up => (bot[0], bot[1] - 1),
//             Direction::Down => (bot[0], bot[1] + 1),
//             Direction::Left => (bot[0] - 1, bot[1]),
//             Direction::Right => (bot[0] + 1, bot[1]),
//             _ => (bot[0], bot[1]),
//         };
//         return *dir;
//         if !memory.is_visited(next_pos) {}
//     }

//     Direction::Wait
// }
// fn move_towards(c_pos: &(i32, i32), t_pos: (i32, i32), state: &GameState) -> Direction {
//     let (cx, cy) = *c_pos;
//     let (tx, ty) = t_pos;

//     // Links
//     if tx < cx && !state.is_wall([cx - 1, cy]) {
//         return Direction::Left;
//     }

//     // Rechts
//     if tx > cx && !state.is_wall([cx + 1, cy]) {
//         return Direction::Right;
//     }

//     // Oben
//     if ty < cy && !state.is_wall([cx, cy - 1]) {
//         return Direction::Up;
//     }

//     // Unten
//     if ty > cy && !state.is_wall([cx, cy + 1]) {
//         return Direction::Down;
//     }

//     // Sollte nie passieren
//     eprintln!("UNEXPECTED BEHAVIOR");
//     Direction::Wait
// }
