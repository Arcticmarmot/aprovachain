#![no_main]
#![no_std]
extern crate alloc;

use alloc::string::String;
use risc0_zkvm::guest::env;
use aprova_core::CheckResult;
risc0_zkvm::guest::entry!(main);

fn main() {
    let s: String = env::read();
    let flag;
    if s.contains("secret") {
        flag = true;
    } else {
        flag = false;
    }
    let result = CheckResult {
        input: s,
        result: flag
    };
    env::commit(&result);
}
