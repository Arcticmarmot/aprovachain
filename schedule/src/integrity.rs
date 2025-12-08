
#[derive(Debug)]
pub struct Integrity {
    pub score: u32
}

impl Default for Integrity {
    fn default() -> Self {
        Self {
            score: 100
        }
    }
}