use std::io::{self, BufRead, Write};

mod astar;
mod decision;
mod explore;
mod helper_func;
mod highlights;
mod memory;
mod signals;
mod signal_manager;
mod states;

use decision::decide;
use states::GameState;

use crate::memory::BotMemory;

use crate::highlights::Highlighter;

fn main() {
    eprint!("A small step for the bot, but a giant leap for the developers");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut memory = BotMemory::new();

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

        // TODO: CONFIG STATT IN GAMESTATE, IN BOT MEMORY SCHREIBEN
        if let Some(cfg) = &state.config {
            memory.config.width = cfg.width;
            memory.config.height = cfg.height;
            memory.config.vis_radius = cfg.vis_radius;
            memory.config.max_ticks = cfg.max_ticks;
            memory.config.gem_ttl = cfg.gem_ttl;
            memory.config.signal_radius = cfg.signal_radius;
            // memory.config.signal_radius = cfg.signal_radius; // Falls du das separat noch brauchst // ne danke abi brauch ich nicht
            eprintln!(
                "Width: {}, Height: {}",
                memory.config.width, memory.config.height
            );
        }

        memory.update(&state);

        // Debugging: Path highlighten
        let mut highlights = Highlighter::new();

        let dir = decide(&state, &mut memory, &mut highlights);

        memory.last_pos = Some(state.bot); // get last pos

        let highlights_json = highlights.to_json();
        println!("{} {}", dir.as_geo(), highlights_json);
        stdout.flush().unwrap();
    }
}

// BEKANNTE BUGS:
// PROBLEMATISCHE SEEDS:

// FIXME:
