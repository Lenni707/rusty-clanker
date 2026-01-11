use serde_json::json;
use std::io::{self, BufRead, Write};

mod decision;
mod memory;
mod states;
mod signal;
mod explore;
mod helper_func;
mod astar;
mod highlights;

use decision::decide;
use states::GameState;

use crate::memory::BotMemory;

use crate::highlights::Highlighter;

fn main() {
    eprint!("A small step for the bot, but a giant leap for the developers");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut memory = BotMemory::new();
    let mut width = memory.width; //100;
    let mut height = memory.height; //100;

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
            memory.width = cfg.width;
            memory.height = cfg.height;
            memory.signal_radius = cfg.signal_radius;
            eprintln!("Width: {}, Height: {}", memory.width, memory.height);
        }

        memory.update(&state);

        // Debugging: Path highlighten
        let mut highlights = Highlighter::new();
        
        let dir = decide(&state, &mut memory, width, height, &mut highlights);

        let highlights_json = highlights.to_json();
        println!("{} {}", dir.as_geo(), highlights_json);
        stdout.flush().unwrap();
    }
}

// BEKANNTE BUGS:
// PROBLEMATISCHE SEEDS:

// FIXME: