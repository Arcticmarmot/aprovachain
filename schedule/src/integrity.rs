
#[derive(Debug)]
pub struct Integrity {
    pub score: u128
}

impl Default for Integrity {
    fn default() -> Self {
        Self {
            score: 100
        }
    }
}