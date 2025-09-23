#![no_main]
#![no_std]
extern crate alloc;

use alloc::string::String;
use risc0_zkvm::guest::env;
use aprova_core::{CheckResult, Request};
use serde_json::from_str;

risc0_zkvm::guest::entry!(main);

fn main() {
    let data: String = env::read();
    let request: Request = from_str(&data).unwrap();
    let result = is_approved(&request);
    let result = CheckResult {
        input: request,
        result
    };
    env::commit(&result);
}

/**

*/
fn is_approved(req: &Request) -> bool {
    let mut result = true;
    let weight = req.weight;
    let max_alt = req.max_alt;
    if weight > 1.0 {
        result = false;
    }
    if max_alt > 1000.0 {
        result = false
    }
    result
}
