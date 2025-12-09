
#[derive(Debug)]
pub struct Timeliness {
    pub score: u128
}

impl Default for Timeliness {
    fn default() -> Self {
        Self {
            score: 100
        }
    }
}