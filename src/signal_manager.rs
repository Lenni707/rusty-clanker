use std::collections::HashSet;
use crate::helper_func::Pos;
use crate::signals_solver::SignalSolver;

pub struct SignalManager {
    pub solver: SignalSolver,
    pub initialized: bool,
}

impl SignalManager {
    pub fn new(width: i32, height: i32, signal_radius: f32) -> Self {
        SignalManager { 
            solver: SignalSolver::new(width, height, signal_radius),
            initialized: false,
        }
    }
    
    pub fn add_measurement(
        &mut self, 
        bot_pos: Pos, 
        signal: f32, 
        known_walls: &HashSet<Pos>, 
        _vis_floors: &Vec<Pos>  // Nicht mehr gebraucht
    ) {
        if !self.initialized {
            self.initialized = true;
            eprintln!("Signal Manager initialized");
        }

        self.solver.add_measurement(bot_pos, signal);
    }

    pub fn get_target(&self, known_walls: &HashSet<Pos>) -> Option<Pos> {
        if !self.initialized {
            return None;
        }
        
        self.solver.get_target(known_walls)
    }
}