use serde_json::json;
use std::io::{self, BufRead, Write};

mod decision;
mod memory;
mod states;

use decision::decide;
use states::GameState;

use crate::memory::BotMemory;

fn main() {
    eprint!("A small step for the bot, but a giant leap for the developers");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut memory = BotMemory::new();
    let mut width = 100; // Default einstellungen
    let mut height = 100;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let state: GameState = match serde_json::from_str(&line) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("JSON parse error: {e}");
                println!("WAIT");
                continue;
            }
        };

        // Config speichern, wenn sie vorhanden ist: Da sie nur einmal am Anfang geschickt wird
        if let Some(config) = &state.config {
            width = config.width;
            height = config.height;
            eprintln!("Width: {}, Height: {}", width, height)
        }

        memory.update(&state);

        let (dir, path) = decide(&state, &mut memory, width, height);
        
        // Debugging: Path highlighten
        let mut highlights = Vec::new();

        let mut path_color = "#ff0000";
        if let Some(explore_pos) = memory.exploration_target {
            path_color = "#00ff00";
            highlights.push(json!([explore_pos[0], explore_pos[1], "#00ff00"])); // Nur das Array!
        }
        // Gem Target
        if let Some(gem_target) = &memory.current_target {
            highlights.push(json!([gem_target.position[0], gem_target.position[1], "#ff0000"]));
        } // Path
        if let Some(path) = path {
            for pos in path.iter().skip(1) { // skip(1) um Startposition zu überspringen
                highlights.push(json!([pos[0], pos[1], path_color]));
            }
        }
        let highlights_json = if !highlights.is_empty() {
            json!({"highlight": highlights})
        } else {
            json!({})
        };
        if !highlights.is_empty() {}
        println!("{} {}", dir.as_geo(), highlights_json);
        stdout.flush().unwrap();
    }
}

// BEKANNTE BUGS:
// Beim Exploren in keinen Bereichen ist der Bot stuck: Vielleicht die letzen Schritte zurück gehen? Randam laufen?
// Feature Idee: Beim Exploren nicht vis_radius betrachen, sondern alles, aber entferntere Felder intressanter machen nach Kriterien wie bspw. ob dort eine Wall ist?
// Dieses Feature glaube ich ist redundant und vielleicht gar nicht sooo lokrativ, da neuen Felder so weniger oft gefunden werden
// PROBLEMATISCHE SEEDS:
// 10t0bcc, 5uy32j, hu31cj, 18udp4u, 1vhh6kb, bh34pf, 1iv223z
