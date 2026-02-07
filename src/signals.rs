use crate::helper_func::{Pos, heuristic};
use crate::memory::BotMemory;
use crate::states::GameState;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// weight ist |particle signal sum - bot.signal_level|

pub struct Particle {
    pub gem_positions: Vec<Pos>,
    pub weight: f32,
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

fn gen_random_free_pos(state: &GameState, mem: &BotMemory, rng: &mut StdRng) -> Pos {
    loop {
        let pos = [
            rng.gen_range(0..mem.config.width),
            rng.gen_range(0..mem.config.height),
        ];
        if !state.floor.contains(&pos) && !mem.known_walls.contains(&pos) {
            return pos;
        }
    }
}

pub struct ParticleFilter {
    particles: Vec<Particle>,    // Vec aus Hypothesen
    num_particles: usize,        // wv Partikel
    rng: StdRng,                 // für rng basierend auf dem seed
    jitter_config: JitterConfig, // Erklärung steht bei der implementierung
    width: i32, // Fuck you wenn du sagst das das ineffizient ist weißt du durch wie viele Funktionen in memory durchgeben müsste??
    height: i32, // Das ist jetzt so wenn du damit ein Problem hast dann ist das jetzt wohl so
    free_pos: Vec<Pos>, // Das geht doch sonst nicht für echt, ich speichere die hier damit ich nicht jedes mal ganz BotMen und State übergeben muss
}

impl ParticleFilter {
    // Constructur
    pub fn new(
        num_particles: usize, // wv Partikel
        seed: u64,
        mem: &BotMemory,
    ) -> Self {
        ParticleFilter {
            particles: Vec::new(),
            num_particles,
            rng: StdRng::seed_from_u64(seed),
            jitter_config: JitterConfig::default(),
            width: mem.config.width,
            height: mem.config.height,
            free_pos: Vec::new(),
        }
    }
    // Alle free_pos speichern
    fn save_free_pos(&mut self, state: &GameState, mem: &BotMemory, rng: &mut StdRng) {
        let mut free_pos: Vec<Pos> = Vec::new();
        loop {
            let pos = [
                rng.gen_range(0..mem.config.width),
                rng.gen_range(0..mem.config.height),
            ];
            if !state.floor.contains(&pos) && !mem.known_walls.contains(&pos) {
                free_pos.push(pos);
            }
            if free_pos.len() > 5 {
                return;
            }
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
                weight: 1.0 / self.num_particles as f32,
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

            // Eigentlich gibt es keine Messfehler
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

    //==========================================================================
    // Jitter, sorry das ich soviel Platz verwende, ich wollte das verstehen
    //==========================================================================

    pub fn jitter(&mut self) {
        for particle in &mut self.particles {
            let action: f32 = self.rng.r#gen();

            // Ich entschuldige mich dafür, aber das muss so da ich sonst 2 &mut self.particles habe. So lasse ich das. #FIXME oder nicht idc
            // Die funktionen dürfen self nicht als parameter haben, da sonst das iterieren über &mut self.particles und auch das verändern von particle
            // in den fn 2 mutable Referencen sind. Das ist sehr traurig aber da müssen wir jetzt durch wir gewinnen duch einsammeln nicht durch schönen code.
            if action < self.jitter_config.move_prob {
                Self::move_gem(
                    particle,
                    self.jitter_config.move_range,
                    self.width,
                    self.height,
                    &mut self.rng,
                );
            } else if action < self.jitter_config.move_prob + self.jitter_config.add_prob {
                Self::add_random_gem(particle, self.free_pos.clone(), &mut self.rng);
            } else {
                Self::remove_random_gem(particle, &mut self.rng);
            }
        }
    }

    // HILFSFUNKTIONEN FÜR DIE JITTER SACHEN:
    // Ändert die config, kann später verwendet werden wenn bspw zu verschiedenen Zeiten andere Werte verwerndet werden sollen
    pub fn set_jitter_config(&mut self, config: JitterConfig) {
        self.jitter_config = config;
    }
    pub fn move_gem(
        particle: &mut Particle,
        range: i32,
        width: i32,
        height: i32,
        rng: &mut StdRng,
    ) {
        if particle.gem_positions.is_empty() {
            return;
        }
        let gem_index = rng.gen_range(0..particle.gem_positions.len());
        let move_x = rng.gen_range(-range..range);
        let move_y = rng.gen_range(-range..range);

        let old_pos = particle.gem_positions[gem_index];
        let new_x = (old_pos[0] + move_x).clamp(1, width - 1); // TODO: Maybe sind die Werte hier shit, aber sie sind glaub ich schon logisch
        let new_y = (old_pos[1] + move_y).clamp(1, height - 1);

        particle.gem_positions[gem_index] = [new_x, new_y]
    }
    pub fn add_random_gem(particle: &mut Particle, free_pos: Vec<Pos>, rng: &mut StdRng) {
        if particle.gem_positions.len() >= 3 {
            return;
        }

        let free_pos = free_pos[rng.gen_range(0..free_pos.len())]; // OMG ES TUT MIR LEID WAS IST DAS?

        particle.gem_positions.push(free_pos);
    }
    pub fn remove_random_gem(particle: &mut Particle, rng: &mut StdRng) {
        if particle.gem_positions.len() <= 1 {
            return;
        }
        let gem_idx = rng.gen_range(0..particle.gem_positions.len());

        particle.gem_positions.remove(gem_idx);
    }
}

//==========================================================================
// Jitter Config
//==========================================================================
pub struct JitterConfig {
    pub move_prob: f32,   // Wahrscheinlickeit fürs verschieben
    pub add_prob: f32,    // wahrscheinlichkeit das hinzugefügt wird
    pub remove_prob: f32, // wahrscheinlichkeit das gelöscht wird
    pub move_range: i32,  // Distanz, wie weit der gem bewegt wird
}
impl Default for JitterConfig {
    // FIXME: Die Werte hier sind geraten
    fn default() -> Self {
        JitterConfig {
            move_prob: 0.7,
            add_prob: 0.15,
            remove_prob: 0.15,
            move_range: 2,
        }
    }
}

// -----------------------------------------------------------------
// Hilfsfunktionen
// -----------------------------------------------------------------

pub fn calc_dist_for_level(signal: f32, radius: f32) -> f32 {
    radius * (1. / signal - 1.).sqrt()
}

pub fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = heuristic(pos1, pos2);
    1.0 / (1. + (dist as f32 / radius).powf(2.))
}
