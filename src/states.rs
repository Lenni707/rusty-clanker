use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GameState {
    pub config: Option<Config>,
    pub tick: u32,
    pub bot: [i32; 2],
    pub wall: Vec<[i32; 2]>,
    pub floor: Vec<[i32; 2]>,
    pub visible_gems: Vec<Gem>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub width: i32,
    pub height: i32,
    pub vis_radius: i32,
    pub max_ticks: u32,
    pub gem_ttl: u32,
}

#[derive(Debug, Deserialize)]
pub struct Gem {
    pub position: [i32; 2],
    ttl: u32,
}

impl GameState {
    pub fn is_wall(&self, pos: [i32; 2]) -> bool {
        self.wall.contains(&pos)
    }
}
