#![no_main]
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);
fn main() {
    let data: Vec<u8> = env::read();
    env::log("pass");
    env::commit(&data);
}