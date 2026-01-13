use crate::states::GameState;
use crate::memory::BotMemory;
use crate::helper_func::{euclidean, get_neighbours, Pos, is_free, clamp_pos};

fn _calc_signal_at_pos(radius: f32, pos_1: Pos, pos_2: Pos) -> f32 {
    let distance = euclidean(pos_1, pos_2);
    let signal = 1.0 / (1.0 + (distance * distance) / (radius * radius));
    signal
}

fn add(a: Pos, b: Pos) -> Pos { [a[0] + b[0], a[1] + b[1]] }
fn sub(a: Pos, b: Pos) -> Pos { [a[0] - b[0], a[1] - b[1]] }

fn left(dir: Pos) -> Pos { // relatives turnen, also je nach dir
    match dir {
        [1, 0] => [0, -1],
        [-1, 0] => [0, 1],
        [0, 1] => [1, 0],
        [0, -1] => [-1, 0],
        _ => [0, 0],
    }
}
fn right(dir: Pos) -> Pos { left(left(left(dir))) }
fn back(dir: Pos) -> Pos { [-dir[0], -dir[1]] }

/// felder um einen rum proben
fn probe_step(memory: &BotMemory, current: Pos) -> Option<Pos> {
    let prev = memory.last_pos?;
    let dir = sub(current, prev);

    let candidates = [left(dir), right(dir), back(dir)]; 

    // lieber nicht einfach auf vorheriges feld gehen sonder erstmal recht links test
    for step in candidates {
        let n = add(current, step);
        if !is_free(memory, n) { continue; }
        if n == prev { continue; }
        return Some(n);
    }

    // wenn nur zurück geht => zurück gehen
    if is_free(memory, prev) { Some(prev) } else { None }
}

/// move towards stronger signal, if not improving => probe_step / move back (included in probe step)
pub fn move_to_signal(memory: &mut BotMemory, state: &GameState) -> Pos {
    if let Some(target) = memory.signal_target.take() {
        return target;
    }

    let current = state.bot;
    let current_signal = state.signal_level;

    let delta_signal = match memory.last_signal {
        Some(prev_signal) => current_signal - prev_signal,
        None => 0.0,
    };

    let going_up = delta_signal > 0.0;

    // bisschen deppert aber die Richtung, in die gegangen wurde (mir ist nichts besseres eingefallen)
    let dir: Option<Pos> = memory.last_pos.map(|prev| sub(current, prev));

    let next = if going_up {
        // wenn signal größer wird, eiter in gleiche richtung
        if let Some(d) = dir {
            let forward = add(current, d);
            if is_free(memory, forward) {
                forward
            } else {
                let step = fallback_with_astar(d, state, memory);
                memory.signal_target = Some(step);
                step
            }
        } else {
            // erster move => irgendwo hin TODO: mit Timon abklären das er erst ein Signal anfängt zu chasen, wenn das signal_level ein bestimmtes level erreicht hat
            pick_any_free_neighbour(memory, current)
        }
    } else {
        // signal wird nicht größer => anfangen zu proben
        probe_step(memory, current).unwrap_or_else(|| pick_any_free_neighbour(memory, current))
    };

    next
}

fn pick_any_free_neighbour(memory: &BotMemory, current: Pos) -> Pos {
    for n in get_neighbours(current) {
        if is_free(memory, n) {
            return n;
        }
    }
    current
}

fn calc_dist_with_signal(signal: f32, radius: f32) -> f32 {
    let distance = radius * (1./signal - 1.).sqrt(); // refactored formula for distance based on formula for signal
    distance
}

pub fn fallback_with_astar(dir: Pos, state: &GameState, memory: &BotMemory) -> Pos {
    let dist = calc_dist_with_signal(state.signal_level, memory.config.signal_radius);

    // float dist to resonaböe lookahead
    let mut lookahead = (dist * 0.8).round() as i32; // 0.8 = aim a bit before the peak
    lookahead = lookahead.clamp(6, 20);          // clamp range (tune if you want)

    // das ungefähre goal
    let mut goal: Pos = [
        state.bot[0] + dir[0] * lookahead,
        state.bot[1] + dir[1] * lookahead,
    ];
    goal = clamp_pos(goal, memory.config.width, memory.config.height); // falls goal out of map wäre => clampen

    goal
}

// TODO:
// 3) When the signal goes down, your probing is too weak in mazes

// Your probe order is [left, right, back]. That’s okay, but in a maze you can get stuck “sliding” and not escaping.

// Make it maze-smart:

// track stuck_ticks (how many ticks since signal improved)

// if stuck_ticks gets large, stop local probing and do a bigger detour (A*).

// 4) “Distance from signal” needs clamping (or you’ll aim insanely far)

// If signal is small, dist = r * sqrt(1/s - 1) becomes huge, and your virtual goal will be far away (often the edge of the map). In wall-heavy maps, this creates long, pointless detours.

// Fix: turn it into a lookahead that’s clamped, like:

// minimum 6 tiles

// maximum 20 tiles

// 5) Use A* only when it matters

// On wall-heavy maps, you’ll want to A* more often, but not every tick.

// Simple rule:

// If forward is blocked → A* detour

// If stuck_ticks >= 3 → A* detour

// Otherwise → cheap gradient+probe

// That keeps it fast but not wasteful.

// 6) Avoid ping-pong / loops

// In mazes, a pure “left/right/back” policy can oscillate.

// Fix: keep a tiny “recent positions” list (like last 10 tiles) and avoid stepping into them unless forced. This makes the crab stop pacing in front of a wall.

// What “better in many walls” looks like behavior-wise

// When uphill and open: crab runs straight.

// When uphill but wall blocks: crab immediately A*s around it toward a point ahead.

// When downhill: crab probes left/right, but if it keeps failing: crab A*s to escape the trap.

// Crab avoids bouncing back and forth.

