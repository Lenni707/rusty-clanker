use std::collections::BTreeMap;

use crate::states::GameState;
use crate::memory::BotMemory;
use crate::helper_func::euclidean;

#[derive(Debug, PartialEq)]
pub struct SignalBasedGems{
    pub position: [i32; 2]
}

fn calc_dist_with_signal(signal: f32, radius: f32, start: [i32; 2]) -> f32 {
    let distance = radius * (1./signal - 1.).sqrt(); // refactored formula for distance based on formula for signal
    distance
}

fn calc_signal_at_pos(radius: f32, pos_1: [i32; 2], pos_2: [i32; 2]) -> f32 {
    let distance = euclidean(pos_1, pos_2);
    let signal = 1.0 / (1.0 + (distance * distance) / (radius * radius));
    signal
}

fn get_gem_based_on_signal(){
    
}