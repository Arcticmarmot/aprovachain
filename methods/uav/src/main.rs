#![no_main]
#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
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
    env::log("hello, risc0");
    env::commit(&data);
}