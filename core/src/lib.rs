use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub input: Request,
    pub result: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub model: String,
    pub weight: f32,
    pub coords: Vec<Coord>,
    pub max_alt: f32,
    pub start_utc: String,
    pub end_utc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coord {
    lon: f64,
    lat: f64,
}