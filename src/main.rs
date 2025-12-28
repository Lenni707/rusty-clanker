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

        let dir = decide(&state, &memory, width, height);
        println!("{}", dir.as_geo());
        stdout.flush().unwrap();
    }
}
