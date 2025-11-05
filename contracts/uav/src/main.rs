#![no_main]
#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
// use serde_json::from_str;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CheckResult {
    pub input: Request,
    pub result: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Request {
    pub id: String,
    pub model: String,
    pub weight: f32,
    pub coords: Vec<Coord>,
    pub max_alt: f32,
    pub start_utc: String,
    pub end_utc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Coord {
    lon: f64,
    lat: f64,
}

risc0_zkvm::guest::entry!(main);
fn main() {
    let data: Vec<u8> = env::read();
    // let request: Request = from_str(&data).expect("bad request format");
    // let result = is_approved(&request);
    // let result = CheckResult {
    //     input: request,
    //     result
    // };
    env::commit(&data);
}

// fn is_approved(req: &Request) -> bool {
//     let mut result = true;
//     let weight = req.weight;
//     let max_alt = req.max_alt;
//     if weight > 1.0 {
//         result = false;
//     }
//     if max_alt > 1000.0 {
//         result = false
//     }
//     result
// }
