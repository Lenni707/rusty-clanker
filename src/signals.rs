use crate::helper_func::{Pos, euclidean, get_dist, heuristic};
use crate::astar::astar;
use crate::highlights::{self, Highlighter, colors};
use crate::portals::Portal;

use std::collections::{BTreeMap, HashMap, HashSet};
use crate::memory::ChannelInfo;

// Meine Idee wie ich das machen würde: 
// Wir lassen die Grundstuktur hier wie sie ist, wir brauchen ja aber an sich nur noch die "find_single_gem", denn:
// - Wir haben die channels für jeden Gem. Jeden Frame gehen wir alle channels durch und gucken wie sicher wir uns sind, das wir den 
//   gem in diesem Channel gefunden haben.
// - Wir nehmen uns den Gem, bei dem wir uns am sichersten sind und der nah dran ist
//      - Theoretisch können wir auch noch einen extra check machen mit dem gesamten Signal-level, vielleicht wenn es sehr hoch ist oder so
// Das einzige was wir eigentlich machen müssen ist einen Filter einbauen, da es ja das signal noise gibt. Wir wissen wie stark das signal noise ist,
// damit können wir errechnen wie stark das noise ungefiltern sein müsste, und damit ungefähre messungen machen.

// In SigSolcer würde ich measurements zu einem Vec<Pos, Vec<f32>> machen, dann dadurch durchgehen. Das ist eigentlich machbar glaube ich.

// Generell sollten wir aber wenn wir lust haben mal ein bisschen was erneuern, was ist den bitte decide.rs???? Und memory ist auch grauenhaft!
// as wahre Recht. Man darf dieser Partei nicht nger dabel zusehen, wie sie die Demokratie von ihr übrigist.
// so lange für sich vereinnahmt, bis nichts mehr
// Eva Ricarda Lautsch, Ein Verbot der AfD ist überfallig, in: https://www.zeit.de/politik/deutschland/2024-10/ afd-verbot-verfahren-demokratie-gefahr, 15.10.2024
pub struct SignalSolver {
    measurements: Vec<(Pos, BTreeMap<usize, ChannelInfo>)>, // usize ist der index des channels, bist erlergisch gegen Kommentare oderß
    refinded_measurments: Vec<(Pos, BTreeMap<usize, ChannelInfo>)>, // idee: Die measurments nehmen, und dann rückwürkend auch bearbeiten wenn es neue Infos gibt
    guesses_for_channels: HashMap<usize, Option<(Pos, f32)>>, // die Position und einen confidence Wert, wie sicher der Gem da ist.
    width: i32,
    height: i32,
    signal_radius: f32,
    signal_noise: f32,
    signal_fade: f32, 
    pub bot_pos: Pos,
    pub known_walls: HashSet<Pos>
}

impl SignalSolver {
    pub fn new(width: i32, height: i32, signal_radius: f32, bot_pos: Pos, signal_noise: f32, known_walls: HashSet<Pos>, signal_fade: f32) -> Self {
        SignalSolver {
            width,
            height,
            refinded_measurments: Vec::new(),
            guesses_for_channels: HashMap::new(),
            signal_radius,
            signal_fade,
            measurements: Vec::new(),
            signal_noise,
            bot_pos: bot_pos,
            known_walls: known_walls
        }
    }
    pub fn add_measurement(&mut self, channels: BTreeMap<usize, ChannelInfo>, pos: Pos) {
        if self.measurements.len() > 25 { // getestet// TODO: Diese Nummer ausprobieren was am besten ist.
            self.measurements.remove(0);
        }
        
        self.measurements.push((pos, channels.clone()));
    }

    // man eigentlich ist das so geil
    //pub fn update_measurenments(&mut self, target_gem: Pos, radius: f32) {
    //    for (bot_pos, channels) in &mut self.measurements {
    //        let contrib = signal_from_dist(*bot_pos, target_gem, radius);
    //        *signal = (*signal - contrib).max(0.0);

    //    }
    //}

    pub fn clear_measuremnts(&mut self) {
        self.measurements.clear();
    }

    pub fn clear_channel_measurement_by_index(&mut self, index: usize) {
        for i in 0..self.measurements.len() {
            if let Some(channel) = self.measurements[i].1.get_mut(&index) {
                channel.signal = 0.0;
                channel.age = 0;
                channel.added_fade_impact = 0.0;
            }
        }
    }

    pub fn get_index_of_collected_gem(&mut self) -> Option<usize> {
        if self.measurements.len() < 2 {
            return None;
        }
        let current_measurement = &self.measurements[self.measurements.len() - 1].1;
        let past_measurment = &self.measurements[self.measurements.len() - 2].1;

        for i in 0..past_measurment.len() {
            if let Some(past_maes) = past_measurment.get(&i) {
                if past_maes.signal == 0.0 {
                    continue;
                }
                if let Some(curr_meas) = current_measurement.get(&i) {
                    if curr_meas.signal == 0.0 {
                        return Some(i);
                    }
                }
            }
        }
        None
    }
    
    // berechnet wie gut die position zu dem gemessen signal wert passt und gibt einen score
    fn rate_position(&self, gem_positions: &[Pos], channel_num: usize) -> f32 {
        let mut total_error = 0.0;
        let mut weight_sum = 0.0;

        for (bot_pos, channel) in &self.measurements {
            if let Some(info) = channel.get(&channel_num) {
                if info.signal == 0.0 { continue; }

                let fade_factor = if self.signal_fade > 0.0 && info.age > 0 {
                    let age = info.age as f32;
                    let fade = self.signal_fade;
                    // Fade-in: erste `fade` Ticks
                    (age / fade).min(1.0)
                } else {
                    1.0
                };

                if fade_factor < 0.05 { continue; } // zu unsicher

                // Signal auf "volles" Signal hochrechnen – aber pro Messung einzeln
                let corrected_signal = info.signal / fade_factor;

                let mut expected = 0.0;
                for gem_pos in gem_positions {
                    expected += signal_from_dist(*gem_pos, *bot_pos, self.signal_radius);
                }

                let error = (expected - corrected_signal).abs();
                // Gewichtung: Messungen mit höherem fade_factor sind zuverlässiger
                let weight = fade_factor * fade_factor;
                total_error += error * weight;
                weight_sum += weight;
            }
        }

        if weight_sum == 0.0 { return 0.0; }
        1.0 / (0.001 + total_error / weight_sum)
    }

    pub fn find_single_gem(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channel_num: usize) -> Option<Pos> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut best_score = 0.0;
        let mut best_pos = None;

        for x in 0..self.width {
            for y in 0..self.height {
                let pos = [x, y];

                if known_walls.contains(&pos) || vis_floors.contains(&pos) {
                    continue;
                }

                let score = self.rate_position(&[pos], channel_num);
                
                if score > best_score {
                    best_score = score;
                    best_pos = Some(pos);
                }
            }
        }
        
        best_pos
    }

    // pub fn get_test_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channels: &Vec<f32>) -> Option<Pos> { // outdated
    //     let mut highest_sig: f32 = 0.0;
    //     let mut best_channel: usize = 0;
    //     for i in 0..channels.len() {
    //         let sig = channels[i];
    //         if sig > highest_sig {
    //             highest_sig = sig;
    //             best_channel = i;
    //         }
    //     }

    //     eprintln!("{}", highest_sig);
    //     let target = self.find_single_gem(known_walls, vis_floors, best_channel);
        
    //     target
    // }

    pub fn get_signal_target(&self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channels: &BTreeMap<usize, ChannelInfo>, forbidden_indexes: &Vec<usize>, portals: &Vec<Portal>) -> Option<(usize, Pos)> {
        // Alle Kandidaten aus den channels entnehmen
        let mut candidates = Vec::new();

        for i in 0..channels.len() {
            if forbidden_indexes.contains(&i) {
                continue;
            }
            if let Some(channel) = channels.get(&i) {
                if channel.signal == 0.0 {
                    continue;
                }
                // if channel.age < self.signal_fade as usize {
                //     continue;
                // }
                if let Some((pos, confidence)) = self.find_single_gem_with_confidence(known_walls, vis_floors, i) {
                    candidates.push((pos, i, confidence));
                }
            }
        }

        let gem_positions: Vec<Pos> = candidates.iter().map(|(pos, _, _)| *pos).collect();

        let mut best_score = f32::NEG_INFINITY;
        let mut best: Option<(usize, Pos)> = None;
        let mut best_idx: Option<usize> = None;

        for (pos, i, confidence) in candidates {
            if let Some(dist) = get_dist(pos, self.bot_pos, &self.known_walls, self.width, self.height, portals) {
                let cluster = self.cluster_score(pos, &gem_positions, 5.0);
                let score = (cluster * 3.0) - (dist as f32 * 0.5) - (1.0 / confidence) * 3.0; //TODO: Hier alle Werte nochmal probieren 
                
                // eprintln!("score {} \n cluster {} \n dist {} \n ====", score, cluster, dist); // DAS HIER KANN AUCH AN SEIN IST eigenltich gant cool
                
                if score > best_score {
                    best_score = score;
                    best = Some((i, pos));
                    best_idx = Some(i);
                }
            }
        }
        if let Some(best_idx) = best_idx {
            let option = self.find_single_gem_with_confidence(known_walls, vis_floors, best_idx);
            if let Some((_, confi)) = option {
                //eprintln!("confidence {}", confi)
            }
        }
        best
    }

    fn cluster_score(
        &self, 
        center: Pos, // der aktuell beobachtete gem von get_sig_target
        all_guesses: &Vec<Pos>, // alle guesses für gems
        cluster_radius: f32 // wie weit soll das ausgeglilchen werden
    ) -> f32 {
        let mut count = 0.0;

        for other_pos in all_guesses {

            if *other_pos == center {
                continue;
            }

            if euclidean(center, *other_pos) < cluster_radius {
                count += 1.0;
            }
        }
        count
    }


    fn find_single_gem_with_confidence(
        &self, known_walls: &HashSet<Pos>, vis_floors: &Vec<Pos>, channel_num: usize
    ) -> Option<(Pos, f32)> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut scores: Vec<([i32; 2], f32)> = Vec::new();

        for x in 0..self.width {
            for y in 0..self.height {
                let pos = [x, y];

                if known_walls.contains(&pos) || vis_floors.contains(&pos) {
                    continue;
                }

                let score = self.rate_position(&[pos], channel_num);
                scores.push((pos, score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let (best_pos, best_score) = scores.first()?;
        let second_score = scores.get(1).map(|(_, s)| *s).unwrap_or(0.0);

        // Wie viel besser ist der erste score als der 2.?
        // let confidence = if second_score > 0.0 {
        //     (best_score / second_score - 1.0).clamp(0.0, 1.0)
        // } else {
        //     1.0
        // };
        let top_positions: Vec<Pos> = scores.iter().take(10).map(|(p, _)| *p).collect();
        let spread = top_positions.iter().map(|p| euclidean(*p, *best_pos)).sum::<f32>() / top_positions.len() as f32;
        let confidence = 1.0 / (1.0 + spread / 3.0); // je kleiner der spread, desto höher confidence
        // eprintln!("raw best_score: {}", confidence);
        Some((*best_pos, confidence))
    }

    // Erstmal nur Test; Idee, das jeden Tick für jeden aktiven Channel ausgerechnet wird wo 
    pub fn guess_all_positions(&mut self, channels: &BTreeMap<usize, ChannelInfo>, vis_floors: &Vec<Pos>, highlights: &mut Highlighter) {
        for i in 0..=channels.len() {
            let channel_info = &channels.get(&i);
            if let Some(channel_info) = channel_info {
                if channel_info.signal == 0. {
                    self.guesses_for_channels.insert(i, None);
                    continue;
                }
                if let Some(info) = self.find_single_gem_with_confidence(&self.known_walls, vis_floors, i) {
                    let pos = info.0;
                    let confidence = info.1;

                    self.guesses_for_channels.insert(i, Some(info));
                }
            }
        }
        for guess in &self.guesses_for_channels {
            if let Some(guess) = guess.1 {
                let pos = guess.0;
                highlights.add_point(pos, colors::YELLOW);
            }
        }
    }
}

// calc the signal level from a euclidian distance
fn signal_from_dist(pos1: Pos, pos2: Pos, radius: f32) -> f32 {
    let dist = euclidean(pos1, pos2);
    let s = 1.0 / (1.0 + (dist / radius).powf(2.0));

    let factor = 1_000_000.0_f32;
    (s * factor).trunc() / factor
}
