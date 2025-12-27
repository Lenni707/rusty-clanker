use std::collections::HashSet;

pub struct BotMemory {
    visited_recently: HashSet<(i32, i32)>,
    last_pos: (i32, i32),
}

impl BotMemory {
    pub fn new() -> Self {
        BotMemory {
            visited_recently: HashSet::new(),
            last_pos: (0, 0),
        }
    }
    pub fn mark_visited(&mut self, pos: (i32, i32)) {
        self.visited_recently.insert(pos);
    }
    pub fn is_visited(&self, pos: (i32, i32)) -> bool {
        self.visited_recently.contains(&pos)
    }
}
