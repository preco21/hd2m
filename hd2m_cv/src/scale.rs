use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct AutoScale {
    original_size: (usize, usize),
    target_size: (usize, usize),
    map: BTreeMap<String, usize>,
}

impl AutoScale {
    pub fn new(width: i32, height: i32, scale: f64) -> Self {
        Self {
            width,
            height,
            scale,
        }
    }

    pub fn clone_with_target_size(&self) -> Self {
        Self {
            original_size: self.original_size,
            target_size: self.target_size,
            map: self.map.clone(),
        }
    }

    pub fn scale(&self, x: i32, y: i32) -> (i32, i32) {
        (x + self.width, y + (self.height as f64 * self.scale) as i32)
    }
}
