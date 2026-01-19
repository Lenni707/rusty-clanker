use crate::{memory::BotMemory, states::GameState, helper_func::*};

use std::{
    collections::HashSet,
    i32,
};

// =====================================================
// EXPLORATION von unbekannten Orten (spooky)
// =====================================================

pub fn get_exploration_target(
    state: &GameState,
    memory: &BotMemory,
) -> Option<[i32; 2]> {
    // Verwendet eine Liste aus kürzlich besuchten tiles, um neune zu unbekannt Tiles zu besuchen
    let curr_tick = state.tick;
    let curr_floor_count = state.floor.len();
    let cooldown_duration = 100; // Wenn abortet wurde, wie lange darf das Feld nicht besucht werden?

    let mut candidates = Vec::new();

    // Frontiers holen, also Positionen mit vielen Nachbarn
    let frontiers = get_frontiers(memory);

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
        if let Some(aborted_tick) = memory.aborted_pos.get(&floor_pos) {
            if curr_tick - aborted_tick < cooldown_duration {
                continue;
            }
        }
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

        let spawn_bonus = memory.gem_spawn_heatmap.get(&floor_pos).unwrap_or(&0) * 25;
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

pub fn get_frontiers(memory: &BotMemory) -> Vec<[i32; 2]> {
    // Findet Felder mit unbekannten nachbarn
    let mut frontiers = Vec::new();

    for &known_pos in memory.known_positions.keys() {
        // Alle Nachbarn prüfen
        for [dx, dy] in [[0, 1], [0, -1], [1, 0], [-1, 0]] {
            let neighbor = [known_pos[0] + dx, known_pos[1] + dy];

            // Bounds Check
            if neighbor[0] < 0 || neighbor[0] >= memory.config.width || neighbor[1] < 0 || neighbor[1] >= memory.config.height {
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
// Exploration Abbruch
// =====================================================
pub fn should_abort_exploration(
    state: &GameState,
    target: [i32; 2],
    memory: &BotMemory,
    vis_range: i32
) -> bool {
    // if state.tick < 200 { 
    //     return false;
    // }
    let distance = heuristic(state.bot, target);

    // Wenn die Entfernug größer ist als die vis_range, dann sollte weiter gegangen werden
    if distance > vis_range * 2 {
        return false; // FIXME: Hier könnten Fehler auftregen, da die dist nicht die Wände mit einbezieht. Das klingt für mich aber nicht schlimm, da der Felher dann später gefunden wird.
    }
    
    let tiles_at_target = get_visible_tiles_from_pos(
        target, 
        &memory.known_walls, 
        vis_range, 
        memory.config.width, 
        memory.config.height
    );

    let tiles_now = get_visible_tiles_from_pos(
        state.bot, 
        &memory.known_walls, 
        vis_range, 
        memory.config.width, 
        memory.config.height,
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
        // FIXME: memory.aborted_pos.insert(target, state.tick);
        return true;
    }
    
    false // Sollte nicht abgebrochen werden
}

pub fn get_visible_tiles_from_pos(
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

pub fn has_line_of_sight(
    from: [i32; 2],
    to: [i32; 2],
    walls: &HashSet<[i32; 2]>,
    vis_range: i32
) -> bool {
    // TODO: Ein richtiges Line of Sight hier einbauen, erstmal tue ich so, als ob alles in meiner vis - 1 gesehn wird
    // https://www.redblobgames.com/articles/visibility/ das ist gut
    let dist = heuristic(from, to);

    // Wenn außerhalb der render distance wird nicht zu sehen sein
    if dist > vis_range + 5 {
        return false;
    }
    // Gleiche Position ist immer sichtbar
    if from == to {
        return true;
    }

    let dx = from[0] + to[0];
    let dy = from[1] + to[1];

    // Anzahl der Schritte = Maximalen Koordanatendifferenz
    let steps = dx.abs().max(dy.abs());

    // Schrittgrößen
    let x_steps = dx as f32 / steps as f32;
    let y_steps = dy as f32 / steps as f32;

    // Nummers als floats nehmen, 
    let mut x = from[0] as f32;
    let mut y = from[1] as f32;

    for _ in 0..=steps {
        x += x_steps;
        y += y_steps;

        let pos = [x.round() as i32, y.round() as i32];

        if walls.contains(&pos) {
            return false;
        }
    }
    true
}