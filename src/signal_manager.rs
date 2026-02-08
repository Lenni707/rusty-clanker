use std::collections::HashSet;

use crate::{helper_func::Pos, memory::{BotMemory}, signals::*, states::GameState};

pub struct SignalManager {
    pub filter: ParticleFilter,
    pub past_levels: Vec<(Pos, f32)>,
    pub intitialized: bool,
}

impl SignalManager {
    pub fn new(width: i32, height: i32, signal_radius: f32) -> Self {
        SignalManager { 
            filter: ParticleFilter::new(1000, 42, width, height, signal_radius), 
            past_levels: Vec::new(), 
            intitialized: false 
        }
    }
    
    pub fn initialize_sig_man(&mut self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>) {
        self.filter.initialize(known_walls, vis_floors);
        self.intitialized = true;
    }

    pub fn add_measurement(
        &mut self, 
        bot_pos: Pos, 
        signal: f32, 
        known_walls: &HashSet<Pos>, 
        vis_floors: &Vec<Pos>
    ) {
        if !self.intitialized {
            self.filter.initialize(known_walls, vis_floors);
            self.intitialized = true;
            eprintln!("Initializing");
        }

        if self.filter.particles.is_empty() {
            eprintln!("CRITICAL ERROR: PARTICELS EMPTY AFTER INITIALSATION")
        }

        self.past_levels.push((bot_pos, signal));

        // Update den Filter mit neuen Inforamtionen
        self.filter.update(bot_pos, signal, );

        // Resample, checking effective sample size
        // Das kommt noch, haben wir nicht implementiert. Deswegen TODO:
        self.filter.resample();

        // adding variaiton
        self.filter.jitter(known_walls, vis_floors);
    }

    // DIE HAUPT API
    pub fn get_target(&self) -> Option<Pos> {
        if !self.intitialized {
            eprintln!("Not initialized");
            return None;
        }
        eprintln!("{:?}", self.filter.get_best_gem_estimate());
        self.filter.get_best_gem_estimate()
    }
}