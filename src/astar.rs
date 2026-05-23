use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    i32,
};

use crate::{helper_func::*, portals::Portal};

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


// 2. Gibt alle Nachbarn zurück, no shit right?
fn get_neighbors(pos: [i32; 2]) -> Vec<Pos> {
    vec![
        [pos[0] + 1, pos[1]], // Rechts (X+1)
        [pos[0] - 1, pos[1]], // Links (X-1)
        [pos[0], pos[1] + 1], // Unten (Y+1)
        [pos[0], pos[1] - 1], // Oben (Y-1)
    ]
}

// =====================================================
// A* Pathfinding: Hauptalgorythmus
// =====================================================

pub fn astar(
    start: [i32; 2],
    goal: [i32; 2],
    walls: &HashSet<[i32; 2]>,
    portals: &Vec<Portal>,
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
        f_cost: heuristic_with_portals(start, goal, portals),
    });
    // Hauptschleife
    // - wenn es einen Knoten zum Untersuchen gibt:
    while let Some(current) = open_set.pop() {
        if current.g_cost > *g_score.get(&current.position).unwrap_or(&i32::MAX) {
            continue
        }

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

        for mut neighbor_pos in get_neighbors(current.position) {
            // Ist der Nachbar im Spielfeld, ist das überhaupt relevant hier?
            if !is_in_bounds(neighbor_pos, width, height) {
                continue;
            }
            // portal entries fire befroe walls bc they are allwoed
            let is_portal_entry = portals.iter().any(|p| p.entry == Some(neighbor_pos));

            // Ist der Nachbar eine Wand?
            if walls.contains(&neighbor_pos) && !is_portal_entry {
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
                let h = heuristic_with_portals(neighbor_pos, goal, portals);
                open_set.push(Node {
                    position: neighbor_pos,
                    g_cost: tentative_g_score,
                    f_cost: tentative_g_score + h, // g + h = f
                });
            }
        }

        // wenn die current pos, ein portal entry ist direkt zum exit teleportieren
        if let Some(portal) = portals.iter().find(|p| p.entry == Some(current.position)) {
            if let Some(exit) = portal.exit_from {
                if is_in_bounds(exit, width, height) { // bisschen dumm aber sicher ist sicher
                    // if teleporting with this portal gives a better route, remember it
                    // einfach selbe logik wie oben mit den neighbour fields nur jetzt halt extra wenn man auf einem portal ist
                    let tentative_g_score = *g_score.get(&current.position).unwrap_or(&i32::MAX);
                    let exit_g_score = *g_score.get(&exit).unwrap_or(&i32::MAX);

                    if tentative_g_score < exit_g_score {
                        came_from.insert(exit, current.position);
                        g_score.insert(exit, tentative_g_score);

                        let h = heuristic_with_portals(exit, goal, portals);
                        open_set.push(Node {
                            position: exit,
                            g_cost: tentative_g_score,
                            f_cost: tentative_g_score + h,
                        });
                    }
                }
            }
        }
    }
    // Es kann aktuell sein, das dieser Fehler angezeigt wird. Das ist erstmal okay, und liegt an der Art wie Frontiers implementiert sind:
    // Sie können gelgentlich auf Feldern sein, welche Walls sind.
    eprintln!("UNEXPECTED BEHAIVIOR: Keinen Weg zum Ziel gefunden");
    None
}