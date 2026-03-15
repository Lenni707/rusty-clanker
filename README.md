# 🦀 RustyClanker

A competitive maze-navigation bot for the [Hidden Gems](https://github.com/hidden-gems) challenge, written in Rust. RustyClanker uses signal triangulation and A\* pathfinding to hunt down gems faster than you can say "divine intellect".

```
╔═══════════════════════════════╗
║  💎  . . # . . . . . # . . . ║
║  . # . . # . 🦀. . . . . . . ║
║  . . . . . . . . # . . # . . ║
║  . # # . . . . . . . . . . . ║
║  . . . . # . . . . . # . . . ║
╚═══════════════════════════════╝
        tracking signal...
```

---

## How it works

RustyClanker has three layers that run every tick:

**1. Memory (`memory.rs`)**
Maintains a persistent map of known floors, walls, and gem positions. Uses flood-fill from the bot's current position to infer walls that are outside the reachable area — no need to see every wall explicitly.

**2. Signal Solver (`signals.rs`)**
When a gem is not directly visible, RustyClanker picks up radio signals emitted by hidden gems. The solver collects up to 15 signal measurements from different positions and scores every possible tile on the map:

```
score(tile) = 1 / (0.001 + weighted_error)
```

Each measurement is fade-corrected individually — the signal fades in over the first `signal_fade` ticks, so raw readings are divided by their fade factor before comparison. Measurements get weighted by how reliable they are (higher fade factor = more weight).

**3. Decision (`decide.rs`)**
Priority order each tick:
1. Known visible gem → navigate directly
2. Signal target → best guess from solver
3. Exploration target → least recently seen area
4. Fallback → move towards center

Pathfinding is A\* with the opponent bot treated as a temporary wall when they're within 3 tiles.

---

## Setup

**Requirements:** Rust (stable), the Hidden Gems runner

```bash
git clone https://github.com/lenni707/rusty-clanker
cd rusty-clanker
ruby runner.rb ../rust-clanker
```

**Run against the random walker:**
```bash
./runner.rb . 
```

**Run head-to-head:**
```bash
./runner.rb . /path/to/other-bot
```

**Profile over 20 rounds:**
```bash
./runner.rb --profile .
```

---

## Project structure

```
src/
├── main.rs          # Entry point, JSON I/O
├── states.rs        # GameState + Config parsing
├── memory.rs        # BotMemory, known map, gem tracking
├── signals.rs       # SignalSolver, position guessing
├── decide.rs        # Decision logic
├── astar.rs         # A* pathfinding
├── explore.rs       # Exploration targeting
├── highlights.rs    # Debug overlays
└── helper_func.rs   # Distance, heuristics, utils
```

---

## Stage 3 — Resonance

Current stage runs until **March 15, 2026** with these settings that matter for RustyClanker:

- `vis_radius: 3–7` — small visibility, signal tracking is critical
- `signal_noise` — normally distributed, corrected per-measurement in the solver
- `signal_fade` — fade-in/out handled with per-tick fade factors
- Channel cooldown after gem collection — solver clears stale measurements automatically
