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
    pub fn get_signal_sum(&self, bot_pos: Pos, signal_radius: f32) -> f32 {
        let mut sum = 0.0;
        for i in &self.gem_positions {
            sum += signal_from_dist(*i, bot_pos, signal_radius)
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
        let pos = [rng.gen_range(0..mem.config.width), rng.gen_range(0..mem.config.height)];
        if !state.floor.contains(&pos) && !mem.known_walls.contains(&pos) {
            return pos;
        }
    }
}

pub struct ParticleFilter {
    particles: Vec<Particle>, // Vec aus Hypothesen
    num_particles: usize, // wv Partikel
    rng: StdRng, // für rng basierend auf dem seed
}

impl ParticleFilter {
    // Constructur
    pub fn new(
        num_particles: usize, // wv Partikel
        seed: u64
    ) -> Self {
        ParticleFilter { 
            particles: Vec::new(), 
            num_particles,
            rng: StdRng::seed_from_u64(seed)
        }
    }
    // Intitialize particles with random gam configuration
    pub fn initialize(&mut self, state: &GameState, mem: &BotMemory) {
        self.particles.clear();
        
        for _ in 0..self.num_particles {
            let num_gems = self.rng.gen_range(1..=3); // Zufällinge anzahl an gems, von 1 bis einschließlich 3
            
            let mut gems = Vec::new();
            
            for _ in 0..num_gems {
                // Zufällinge Positionen für die guesses
                let guessed_pos = gen_random_free_pos(state, mem, &mut self.rng);

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
    pub fn update(&mut self, bot_pos: [i32; 2], observed_signal: f32, mem: &BotMemory) {
        let mut total_weight = 0.0;
        
        // Für jeden Partikel
        for particle in &mut self.particles {
            // Was würde ich in der angenommenen Welt messen
            let expected = particle.get_signal_sum(bot_pos, mem.config.signal_radius);
            
            // Wie gut ist das erwartete im Vergleich zum echten Ergebnis?
            let diff = (expected - observed_signal).abs();

            // Smaller diff = höheres gewicht
            particle.weight = 1.0 / (1.0 * diff * 10.0); // TODO: die 10 kann hier geändert werden
            total_weight += particle.weight;

            // // FIXME: Das hier rausnehem, vielleicht ist das besser ohne? Eigentlich gibt es keine Messfehler
            // // Gaussian Likelihood; Je kleiner die diff, desto besser ist die Likelyhood
            // let sigma = 0.1; // Als tuning parameter
            // let likelihood = (-diff * diff / (2.0 * sigma * sigma)).exp(); // OMG EINE e FUNKTION DE SILVAAA. Lass mal den ln davon nehem OMG
        }
        
        // Normalisieren der weights. Sum(weights) = 1.0
        if total_weight > 0.0 {
            for particle in &mut self.particles {
                particle.weight /= total_weight;
            }
        }
    }

    pub fn resample(&self) {
        return;
    }

    pub fn jitter(&mut self) {
        for particle in &mut self.particles {
            let action: f32 = self.rng.gen_range(0.0..=1.0);

            if action < 0.6 {
                // in 60% der Fälle wird der gem leicht verschoben
                if !particle.gem_positions.is_empty() {
                    
                }
            }
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