#![no_main]
#![no_std]
extern crate alloc;

use alloc::string::String;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    let s: String = env::read();
    if s.contains("secret") {
        env::commit(&true);
    } else {
        env::commit(&false);
    }
}
