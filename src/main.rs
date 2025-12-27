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

        let dir = decide(&state);
        println!("{}", dir.as_geo());
        stdout.flush().unwrap();
    }
}
