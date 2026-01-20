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

    // alle recent tiles erstmal principiel nicht erlauben
    for step in candidates {
        let n = add(current, step);
        if !is_free(memory, n) { continue; }
        if is_recent(memory, n) { continue; }
        return Some(n);
    }

    // recent tiles erlauben, wenn keine andere möglichkeit
    for step in candidates {
        let n = add(current, step);
        if !is_free(memory, n) { continue; }
        return Some(n);
    }

    None
}

/// move towards stronger signal, if not improving => probe_step / move back (included in probe step)
pub fn move_to_signal(memory: &mut BotMemory, state: &GameState) -> Pos {
    if let Some(target) = memory.signal_target {
    if state.bot == target {
        memory.signal_target = None;
    } else {
        return target;
    }
}
    let current = state.bot;
    remember_pos(memory, current);
    let current_signal = state.signal_level;

    let delta_signal = match memory.last_signal {
        Some(prev_signal) => current_signal - prev_signal,
        None => 0.0,
    };

    let going_up = delta_signal > 0.0;

    // bisschen deppert aber die Richtung, in die gegangen wurde (mir ist nichts besseres eingefallen)
    let dir: Option<Pos> = memory.last_pos.map(|prev| sub(current, prev));

    if memory.ticks_since_signal_up >= 5 { // added so if the signal didnt improve
        if let Some(d) = dir {
            let step = fallback(d, state, memory); // fallback
                memory.signal_target = Some(step);
                return step
        }
        memory.ticks_since_signal_up = 0;
    }

    let next = if going_up { // wenn signal größer wird, eiter in gleiche richtung
        if let Some(d) = dir {
            let forward = add(current, d);
            if is_free(memory, forward) { // if tile is free (!wall, !edge) "move" forward
                forward
            } else { // if tile == wall or tile == edge
                probe_step(memory, current).unwrap_or_else(|| pick_any_free_neighbour(memory, current))
                // let step = fallback(d, state, memory); // fallback
                // memory.signal_target = Some(step);
                // step
            }
        } else {
            // erster move => irgendwo hin
            pick_any_free_neighbour(memory, current)
        }
    } else {
        // signal wird nicht größer => anfangen zu proben
        memory.ticks_since_signal_up += 1;
        probe_step(memory, current).unwrap_or_else(|| pick_any_free_neighbour(memory, current))
    };

    next
}

fn pick_any_free_neighbour(memory: &BotMemory, current: Pos) -> Pos {
    // prefer not-recent
    for n in get_neighbours(current) {
        if is_free(memory, n) && !is_recent(memory, n) {
            return n;
        }
    }
    // fallback: allow recent
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

pub fn fallback(dir: Pos, state: &GameState, memory: &BotMemory) -> Pos { 
// TODO: // dir is only straight lines
// i doesnt take into account, if the gem is diagonial from me, so im not always taking the most optimized path
    let dist = calc_dist_with_signal(state.signal_level, memory.config.signal_radius);

    // float dist to resonaböe lookahead
    let mut lookahead = (dist * 0.8).round() as i32; // 0.8 = aim a bit before the peak
    lookahead = lookahead.clamp(6, 25);          // clamp range (tune if you want)

    // das ungefähre goal
    let mut goal: Pos = [
        state.bot[0] + dir[0] * lookahead,
        state.bot[1] + dir[1] * lookahead,
    ];
    goal = clamp_pos(goal, memory.config.width, memory.config.height); // falls goal out of map wäre => clampen

    goal
}

// TODO:
// add stuck ticks counter (ticks since signal hasnt improved by alot)
// proben schlaue machen (auf wände achten), smarte auf die gems laufen



fn remember_pos(memory: &mut BotMemory, p: Pos) {
    // avoid duplicates if you stay in place
    if memory.recent_positions.back().copied() == Some(p) {
        return;
    }

    memory.recent_positions.push_back(p);

    while memory.recent_positions.len() > 5 {
        memory.recent_positions.pop_front();
    }
}

fn is_recent(memory: &BotMemory, p: Pos) -> bool {
    memory.recent_positions.iter().any(|&x| x == p)
}
