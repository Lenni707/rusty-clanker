// Feld fürs highlighten

pub struct Highlighter {
    highlights: Vec<([i32; 2], String)>
}

impl Highlighter {
    fn new() -> Self {
        Highlighter {
            highlights: Vec::new(),
        }
    }

    pub fn add_point(&mut self, pos: [i32; 2], color: &str) {
        self.highlights.push((pos, color.to_string()));
    }

    pub fn add_path(&mut self, path: Vec<[i32; 2]>, color: &str) {
        for pos in path {
            self.add_point(pos, color);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.highlights.is_empty()
    }

    pub fn to_json(&self) -> Value {
        let highlight_array: Vec<Value> = self.highlights
            .iter()
            .map(|(pos, color)| json!([pos[0]. pos[1], color]))
            .collect();

        json!({"highlight": highlight_array})
    }

    pub fn clear(&mut self) {
        self.highlights.clear();
    }
}


// Vordefinierte Farben als Konstanten für bessere Lesbarkeit
pub mod colors {
    pub const RED: &str = "#ff0000";
    pub const GREEN: &str = "#00ff00";
    pub const BLUE: &str = "#0000ff";
    pub const YELLOW: &str = "#ffff00";
    pub const PURPLE: &str = "#ff00ff";
    pub const CYAN: &str = "#00ffff";
    pub const ORANGE: &str = "#ff8800";
    pub const WHITE: &str = "#ffffff";
}