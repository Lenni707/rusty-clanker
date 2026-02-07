use crate::memory::{BotMemory};
use crate::states::GameState;
use crate::helper_func::{ Pos, heuristic };

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// weight ist |particle signal sum - bot.signal_level|

pub struct Particle {
    pub gem_positions: Vec<Pos>,
    pub weight: f32
}

impl Particle {
    pub fn get_signal_sum(bot_pos: Pos, signal_radius: f32) -> f32 {
        let sum = 0.0;
        for i in self.gem_positions {
            sum += signal_from_dist(i, bot_pos, signal_radius)
        }
        sum
    }
}

fn gen_random_free_pos(
    state: &GameState,
    mem: &BotMemory,
    rng: &mut StdRng,
) -> Pos {
    loop {
        let pos = [rng.gen_range(0..state.config.width), rng.gen_range(0..state.config.height)];
        if !state.floor.contains(&pos) && !mem.known_walls.contains(&pos) {
            return pos;
        }
    }
}

pub struct ParticleFilter {
    particles: Vec<Particle>, // Vec aus Hypothesen
    num_particles: usize, // wv Partikel
    state: &GameState,
    memory: &BotMemory,
    rng: StdRng, // für rng basierend auf dem seed
}

impl ParticleFilter {
    // Constructur
    pub fn new(
        num_particle: usize, // wv Partikel
        state: &GameState,
        mem: &BotMemory,
        seed: u64
    ) -> Self {
        ParticleFilter { 
            particles: Vec::new(), 
            num_particles, 
            state,
            memory,
            rng: StdRng::seed_from_u64(seed)
        }
    }
    // Intitialize particles with random gam configuration
    pub fn initialize(&mut self) {
        self.particles.clear();
        
        for _ in 0..self.num_particles {
            let num_gems = self.rng.gen_range(1..=3); // Zufällinge anzahl an gems, von 1 bis einschließlich 3
            
            let mut gems = Vec::new();
            
            for _ in 0..num_gems {
                // Zufällinge Positionen für die guesses
                let guessed_pos = gen_random_free_pos(self.state, self.memory, self.rng);

                gems.push(guessed_pos);
            }
            // speichern der Positonen
            self.particles.push(Particle { 
                gem_positions: gems, 
                weight: 1.0/self.num_particles as f32 
            });
        }
    }

    // Bewertet die Partikel anhand der Messungen
    pub fn update(&mut self, bot_pos: [i32; 2], observed_signal: f32) {
        let mut total_weight = 0.0;
        
        // Für jeden Partikel
        for particle in &mut self.particles {
            // Was würde ich in der angenommenen Welt messen
            let expected = particle.get_signal_sum(bot_pos, self.signal_radius);
            
            // Wie gut ist das erwartete im Vergleich zum echten Ergebnis?
            let diff = (expected - observed_signal).abs();

            // FIXME: Das hier rausnehem, vielleicht ist das besser ohne? Eigentlich gibt es keine Messfehler
            // Gaussian Likelihood; Je kleiner die diff, desto besser ist die Likelyhood
            let sigma = 0.1; // Als tuning parameter
            let likelihood = (-diff * diff / (2.0 * sigma * sigma)).exp(); // OMG EINE e FUNKTION DE SILVAAA. Lass mal den ln davon nehem OMG

            particle.weight = likelihood;
            total_weight += likelihood
        }
        
        // Normalisieren der weights. Sum(weights) = 1.0
        if total_weight > 0.0 {
            for particle in &mut self.particles {
                particle.weight /= total_weight;
            }
        }
    }

    pub fn resample(&self) {
        
    }

    pub fn jitter(&mut self) {
        for particle In &mut self.particles {
            let action: f32 = self.rng.gen()
        }
    }
}


// -----------------------------------------------------------------
// Hilfsfunktionen
// -----------------------------------------------------------------
pub fn calc_dist_for_level(signal: f32, radius: f32) -> f32 {
    radius * (1./signal - 1.).sqrt()
}

pub fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = heuristic(pos1, pos2);
    1.0 / (1. + (dist as f32 / radius).powf(2.))
}