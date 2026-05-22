use crate::helper_func::*;

pub struct Portal {
    pub id: usize,
    pub entry: Option<Pos>,
    pub entry_from: Option<Pos>,
    pub exit: Option<Pos>,
    pub exit_from: Option<Pos>,
}

impl Portal {
    pub fn is_complete(&self) -> bool {
        self.entry.is_some() && self.exit.is_some()
    }
}