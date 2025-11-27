#![no_main]
#![no_std]
extern crate alloc;
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

}

risc0_zkvm::guest::entry!(main);
fn main() {
    let data: Vec<u8> = env::read();
    env::log("hello, risc0");
    env::commit(&data);
}