use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GameState {
    pub config: Option<Config>,
    pub tick: u32,
    pub bot: [i32; 2],
    pub wall: Vec<[i32; 2]>,
    pub floor: Vec<[i32; 2]>,
    pub visible_gems: Vec<Gem>,
    pub visible_bots: Option<Vec<EnemyBot>>,
    pub signal_level: f32,
    pub channels: Vec<f32>,
    pub portals: Vec<[i32; 2]>,
    pub portal_stubs: Vec<[i32; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub width: i32,
    pub height: i32,
    pub vis_radius: i32,
    pub max_ticks: u32,
    pub max_gems: u32,
    pub gem_spawn_rate: f32,
    pub gem_ttl: u32,
    pub signal_radius: f32,
    pub signal_fade: f32,
    pub signal_noise: f32,
    pub max_portals: i32,
}

#[derive(Debug, Deserialize)]
pub struct Gem {
    pub position: [i32; 2],
    pub ttl: i32,
}

#[derive(Debug, Deserialize)]
pub struct EnemyBot {
    pub position: [i32; 2],
    pub emoji: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            width: 0,
            height: 0,
            vis_radius: 0,
            max_ticks: 0,
            gem_ttl: 0,
            signal_radius: 0.0,
            signal_fade: 0.0,
            signal_noise: 0.0,
            gem_spawn_rate: 0.0,
            max_gems: 0,
            max_portals: 0,
        }
    }
}
