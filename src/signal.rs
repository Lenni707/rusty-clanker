use std::collections::BTreeMap;

use crate::states::GameState;

use crate::memory::BotMemory;

#[derive(Debug, PartialEq)]
pub struct signal_based_gems{
    pub position: [i32; 2]
}

fn calc_dist_with_signal(signal: f32, radius: f32, start: [i32; 2]) -> f32 {
    let distance = radius * (1./signal - 1.).sqrt(); // refactored formula for distance based on formula for signal
    distance
}