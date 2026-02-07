use crate::{helper_func::Pos, memory::{BotMemory}, signals::*, states::GameState};

pub struct SignalManager {
    pub filter: ParticleFilter,
    pub past_levels: Vec<(Pos, f32)>,
    intitialized: bool,
}

impl SignalManager {
    pub fn new(mem: &BotMemory) -> Self {
        SignalManager { 
            filter: ParticleFilter::new(1000, 1000, mem), 
            past_levels: Vec::new(), 
            intitialized: false 
        }
    }
    pub fn add_measurement(&mut self, bot_pos: Pos, signal: f32, state: &GameState, mem: &BotMemory) {
        if !self.intitialized {
            self.filter.initialize(state, mem);
            self.intitialized = true;
        }

        self.past_levels.push((bot_pos, signal));

        // Update den Filter mit neuen Inforamtionen
        self.filter.update(bot_pos, signal, mem);

        // Resample, checking effective sample size
        // Das kommt noch, haben wir nicht implementiert. Deswegen TODO:

        self.filter.resample();

        // adding variaiton
        self.filter.jitter();
    }

    // DIE HAUPT API
    pub fn get_target(&self) -> Option<Pos> {
        if !self.intitialized {
            return None;
        }
        self.filter.get_best_gem_estimate()
    }
}