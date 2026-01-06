use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    i32,
};

use crate::{memory::BotMemory, states::{self, GameState}};

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
            // Für Determismus
            .then_with(|| self.position[0].cmp(&other.position[0]))
            .then_with(|| self.position[1].cmp(&other.position[1]))
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

pub fn astar(
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
    // Es kann aktuell sein, das dieser Fehler angezeigt wird. Das ist erstmal okay, und liegt an der Art wie Frontiers implementiert sind:
    // Sie können gelgentlich auf Feldern sein, welche Walls sind.
    eprintln!("UNEXPECTED BEHAIVIOR: Keinen Weg zum Ziel gefunden");
    None
}

// =====================================================
// EXPLORATION von unbekannten Orten (spooky)
// =====================================================

fn get_exploration_target(
    state: &GameState,
    memory: &BotMemory,
    width: i32,
    height: i32,
) -> Option<[i32; 2]> {
    // Verwendet eine Liste aus kürzlich besuchten tiles, um neune zu unbekannt Tiles zu besuchen
    let curr_tick = state.tick;
    let curr_floor_count = state.floor.len();
    let mut candidates = Vec::new();

    // Frontiers holen, also Positionen mit vielen Nachbarn
    let frontiers = get_frontiers(memory, width, height);

    let mut all_positions: Vec<[i32; 2]> = Vec::new();

    for &floor_pos in &state.floor {
        all_positions.push(floor_pos);
    }
    for &floor_pos in memory.known_positions.keys() {
        all_positions.push(floor_pos);
    }
    for &floor_pos in &frontiers {
        all_positions.push(floor_pos);
    }
    // Sortieren für Determismus
    all_positions.sort_by_key(|&[x, y]| (x, y));
    all_positions.dedup();

    for &floor_pos in &all_positions {
        let seen = memory.known_positions.get(&floor_pos);

        let is_frontier = frontiers.contains(&floor_pos);

        let (interest_score, exploration_bonus) = match seen {
            None => {
                // Wenn noch nie gesehen
                if is_frontier {
                    (1800, 700) // 500
                } else {
                    (1200, 600)
                }
            }
            Some(&mem) => {
                let age = curr_tick - mem.last_seen_tick;
                if age < 30 {
                    continue; // Kürzlichst besucht, nicht hingehen
                }
                let floor_diff = (curr_floor_count as i32 - mem.vis_floor_count as i32).abs();
                let exploration_bonus = floor_diff * 2;

                (age, exploration_bonus as u32)
            }
        };
        let distance = heuristic(state.bot, floor_pos);

        let spawn_bonus = memory.gem_spawn_heatmap.get(&floor_pos).unwrap_or(&0) * 50;
        // eprintln!("{spawn_bonus}");
        let score =
            interest_score as i32 + exploration_bonus as i32 + spawn_bonus as i32 - distance;

        candidates.push((floor_pos, score));
    }

    candidates.sort_by(|(pos_a, score_a), (pos_b, score_b)| {
        score_b
            .cmp(score_a) // Höherer Score zuerst
            .then_with(|| pos_a[0].cmp(&pos_b[0])) // Bei Gleichstand: x-Koordinate
            .then_with(|| pos_a[1].cmp(&pos_b[1])) // Dann y-Koordinate
    });

    candidates.first().map(|(pos, _)| *pos)
}

pub fn get_frontiers(memory: &BotMemory, width: i32, height: i32) -> Vec<[i32; 2]> {
    // Findet Felder mit unbekannten nachbarn
    let mut frontiers = Vec::new();

    for &known_pos in memory.known_positions.keys() {
        // Alle Nachbarn prüfen
        for [dx, dy] in [[0, 1], [0, -1], [1, 0], [-1, 0]] {
            let neighbor = [known_pos[0] + dx, known_pos[1] + dy];

            // Bounds Check
            if neighbor[0] < 0 || neighbor[0] >= width || neighbor[1] < 0 || neighbor[1] >= height {
                continue;
            }

            let is_unknown = !memory.known_positions.contains_key(&neighbor);
            let is_no_wall = !memory.known_walls.contains(&neighbor);

            if is_no_wall && is_unknown {
                frontiers.push(neighbor);
            }
        }
    }
    frontiers.sort_by_key(|&[x, y]| (x, y));
    frontiers.dedup();
    frontiers
}
// =====================================================
// Bot Exploration Abbruch
// =====================================================
fn should_abort_exploration(
    state: &GameState,
    target: [i32; 2],
    memory: &BotMemory,
    vis_range: i32
) -> bool {
    if state.tick < 200 { eprintln!("öasdfjas"); return false; }
    let distance = heuristic(state.bot, target);

    // Wenn die Entfernug größer ist als die vis_range, dann sollte weiter gegangen werden
    if distance > vis_range * 2 {
        return false; // FIXME: Hier könnten Fehler auftregen, da die dist nicht die Wände mit einbezieht. Das klingt für mich aber nicht schlimm, da der Felher dann später gefunden wird.
    }
    
    let tiles_at_target = get_visible_tiles_from_pos(
        target, 
        &memory.known_walls, 
        vis_range, 
        memory.width, 
        memory.height
    );

    let tiles_now = get_visible_tiles_from_pos(
        state.bot, 
        &memory.known_walls, 
        vis_range, 
        memory.width, 
        memory.height
    );

    let mut new_counter = 0;
    for tile in tiles_at_target {
        let not_yet_visible = tiles_now.contains(&tile);
        if not_yet_visible {
            new_counter += 1;
        }
    }
    if new_counter >= 3 { //TODO: Diese 3 hier ändern und testen
        eprintln!("Aborting Exploration");
        return true;
    }
    
    false // Sollte nicht abgebrochen werden
}

fn get_visible_tiles_from_pos(
    pos: [i32; 2],
    known_walls: &HashSet<[i32; 2]>,
    vis_range: i32,
    width: i32,
    height: i32
) -> HashSet<[i32; 2]> {
    let mut visible = HashSet::new();

    for dx in -vis_range..=vis_range {
        for dy in -vis_range..=vis_range {
            let check_pos = [pos[0] + dx, pos[1] + dy];

            if !is_in_bounds(pos, width, height) {
                continue;
            }

            // Da die vis_range nicht um ein 4 Eck um den Bot ist
            if (dx.abs() + dy.abs()) > vis_range {
                continue;
            }
            if has_line_of_sight(pos, check_pos, known_walls, vis_range) {
                visible.insert(check_pos);
            }
        }
    }
    visible
}

fn has_line_of_sight(
    from: [i32; 2],
    to: [i32; 2],
    walls: &HashSet<[i32; 2]>,
    vis_range: i32
) -> bool {
    // TODO: Ein richtiges Line of Sight hier einbauen, erstmal tue ich so, als ob alles in meiner vis - 1 gesehn wird
    // https://www.redblobgames.com/articles/visibility/ das ist gut
    let dist = heuristic(from, to);
    if dist + 1 < vis_range {
        return true;
    }
    false
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