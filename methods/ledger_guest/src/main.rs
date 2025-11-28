#![no_main]
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use apps::ctr_io::*;
use ledger::call::*;

risc0_zkvm::guest::entry!(main);
fn main() {
    let ctr_input: Vec<u8> = env::read();
    match CtrInput::try_decode_bcs(&ctr_input) {
        Ok(ctr_input) => {
            let context = ctr_input.context;
            let input = ctr_input.input;
            // TODO: 继续完成 ledger 合约
        },
        Err(err) => {
            env::commit(&"error");
            return;
        }
    }
    env::commit(&"ok");
}