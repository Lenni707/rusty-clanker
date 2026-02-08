use std::collections::{HashMap, HashSet};

use crate::helper_func::{Pos, heuristic};
use crate::memory::BotMemory;
use crate::states::GameState;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// weight ist |particle signal sum - bot.signal_level|

#[derive(Clone)]
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
    pub particles: Vec<Particle>,    // Vec aus Hypothesen
    num_particles: usize,        // wv Partikel
    rng: StdRng,                 // für rng basierend auf dem seed
    jitter_config: JitterConfig, // Erklärung steht bei der implementierung
    width: i32, // Fuck you wenn du sagst das das ineffizient ist weißt du durch wie viele Funktionen in memory durchgeben müsste??
    height: i32, // Das ist jetzt so wenn du damit ein Problem hast dann ist das jetzt wohl so
    free_pos: Vec<Pos>, // Das geht doch sonst nicht für echt, ich speichere die hier damit ich nicht jedes mal ganz BotMen und State übergeben muss
    signal_radius: f32,
}

impl ParticleFilter {
    // Constructur
    pub fn new(
        num_particles: usize, // wv Partikel
        seed: u64,
        width: i32,
        height: i32,
        signal_radius: f32,
    ) -> Self {
        ParticleFilter {
            particles: Vec::new(),
            num_particles,
            rng: StdRng::seed_from_u64(seed),
            jitter_config: JitterConfig::default(),
            width: width,
            height: height,
            free_pos: Vec::new(),
            signal_radius: signal_radius,
        }
    }
    // Alle free_pos speichern
    pub fn save_free_pos(&mut self, known_walls: &HashSet<Pos>, known_floor: &HashSet<Pos>, vis_floor: &Vec<Pos>) {
        self.free_pos.clear();
        let max_attempts = 500;
        let target_count = 50;
        
        for _ in 0..max_attempts {
            if self.free_pos.len() >= target_count {
                break;
            }
            
            // Guard gegen width/height = 0
            if self.width <= 0 || self.height <= 0 {
                eprintln!("Invalid config dimensions!");
                break;
            }
            
            let pos = [
                self.rng.gen_range(0..self.width),
                self.rng.gen_range(0..self.height),
            ];
            
            if !vis_floor.contains(&pos) 
                && !known_walls.contains(&pos) 
                && !self.free_pos.contains(&pos) 
            {
                self.free_pos.push(pos);
            }
        }
    }
    // Intitialize particles with random gam configuration
    pub fn initialize(&mut self, known_walls: &HashSet<Pos>, known_floor: &HashSet<Pos>, vis_floor: &Vec<Pos>) {
        self.particles.clear();
        self.save_free_pos(known_walls, known_floor, vis_floor);

        for _ in 0..self.num_particles {
            let num_gems = self.rng.gen_range(1..=3); // Zufällinge anzahl an gems, von 1 bis einschließlich 3
            // TODO: Für die aktuelle Signal sachen: Die Anzahl der etwarteten Gems pro Partikel
            // schätzen und damit dann mehr Partikel mit dieser Menge erstellen

            let mut gems = Vec::new();

            for _ in 0..num_gems {
                // Zufällinge Positionen für die guesses
                let guessed_pos = self.free_pos[self.rng.gen_range(0..self.free_pos.len())]; // ik im sorry

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
    pub fn update(&mut self, bot_pos: [i32; 2], observed_signal: f32) {
        let mut total_weight = 0.0;

        // Für jeden Partikel
        for particle in &mut self.particles {
            // Was würde ich in der angenommenen Welt messen
            let expected = particle.get_signal_sum(bot_pos, self.signal_radius);

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

    pub fn resample(&mut self) {
        if self.particles.is_empty() {
            return;
        }

        let mut new_particles = Vec::new();
        
        // Systematic Resampling (effizienter als naive Methode)
        let mut cumulative = Vec::new();
        let mut sum = 0.0;
        for p in &self.particles {
            sum += p.weight;
            cumulative.push(sum);
        }
        
        // Ziehe num_particles neue Partikel basierend auf Gewichten
        let step = 1.0 / self.num_particles as f32;
        let start: f32 = self.rng.gen_range(0.0..step);
        
        let mut idx = 0;
        for i in 0..self.num_particles {
            let target = start + i as f32 * step;
            
            while idx < cumulative.len() - 1 && cumulative[idx] < target { // only pushes good wheighing particles // index out of bounds
                idx += 1;
            }
            
            new_particles.push(self.particles[idx].clone());
        }
        
        self.particles = new_particles;
        
        // Reset weights
        for p in &mut self.particles {
            p.weight = 1.0 / self.num_particles as f32;
        }
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

        if free_pos.is_empty() {
            eprintln!("Warning: No free positions available for add_random_gem");
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

    //==========================================================================
    // Get the best gem estimate based on average of top cluster
    //==========================================================================
    pub fn get_best_gem_estimate(&self) -> Option<Pos> {
        if self.particles.is_empty() {
            eprintln!("Warning in get_best_gem_estimate: Particles Empty");
            return None;
        }

        // Grid Based with weight
        let grid_size = 3; // TODO: Geratender Wert, der geht besser
        let mut grid_weights: HashMap<(i32, i32), f32> = HashMap::new();

        for particle in &self.particles {
            for gem in &particle.gem_positions {
                let grid_x = gem[0] / grid_size;
                let grid_y = gem[1] / grid_size;
                *grid_weights.entry((grid_x, grid_y)).or_insert(0.0) += particle.weight;
            }
        }

        // Rust ist schon was :/ aber holt die beste Zelle in dem sie mit allen anderen vergleichen wird.
        // Das "?" gibt, sollte unwrap throwen, None aus der Funktion zurück
        let best_cell = grid_weights.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())?;

        // Hier könnten die Koordinaten zurückgegeben werden. Claude empfielt aber das hier noch die genau geschaut wird wo in der Zelle sich die Gems am wahrscheinlichsten befinden
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut total_weight = 0.0;

        for particle in &self.particles {
            for gem in &particle.gem_positions {
                let grid_x = gem[0] / grid_size;
                let grid_y = gem[1] / grid_size;

                // Wenn die grid positionen der vermuteten Gems in der best_cell sind
                if (grid_x, grid_y) == *best_cell.0 {
                    sum_x += gem[0] as f32 * particle.weight; 
                    sum_y += gem[1] as f32 * particle.weight; // Berechnet den gewichteten Durchschnitt in der Zelle, also wo ist der gem in der Zelle gewichtet am wahrscheinlichsten.
                    total_weight += particle.weight;
                }
            }
        }

        if total_weight > 0.0 {
            Some([
                (sum_x / total_weight).round() as i32,
                (sum_y / total_weight).round() as i32,
            ])
        } else {
            None
        }
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
