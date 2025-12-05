#![no_main]
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use apps::ctr_io::{CtrInput, CtrOutcome, CtrOutput, CtrResult, NamespaceKey, WriteSet, CtrContext, ReadSet};
use ledger::call::{address_to_entry_key, LedgerCall};
use anyhow::{anyhow, ensure, Result};
use primitives::hash::{Hash32};
use primitives::trans::{u128_from_be_slice, u128_to_be_vec};

risc0_zkvm::guest::entry!(main);
fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let mut ctx_hash: Hash32 = [0u8; 32];
    let ctr_output: CtrOutput = match CtrInput::try_decode_bcs(&ctr_input_bytes) {
        Ok(ctr_input) => {
            ctx_hash = ctr_input.ctx_hash;
            let context = ctr_input.context;
            let ctr_result = match handle_ledger_call(&context) {
                Ok(ctr_outcome) => {
                    CtrResult::Ok { outcome: ctr_outcome }
                },
                Err(err) => {
                    CtrResult::Err { message: format!("{:#}", err) }
                }
            };
            CtrOutput {
                ctx_hash,
                read_set: context.read_set,
                ctr_result
            }
        },
        Err(err) => {
            CtrOutput {
                ctx_hash,
                read_set: ReadSet::new(),
                ctr_result: CtrResult::Err {
                    message: format!("{:#}", err)
                }
            }
        }
    };
    env::commit(&ctr_output.encode_bcs());
}

fn handle_ledger_call(context: &CtrContext) -> Result<CtrOutcome> {
    let chain_id = context.chain_id;
    let input = &context.input;
    let read_set = &context.read_set;
    let call = LedgerCall::try_decode_bcs(input)?;
    let mut write_set = WriteSet::new();
    let mut answer: Vec<u8> = Vec::new();
    match call {
        LedgerCall::Mint { to, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &to)?;
            let old_bal = load_bal_or_zero(&ns_key, read_set);
            let new_bal = old_bal + amount;
            write_set.insert(ns_key, Some(u128_to_be_vec(new_bal)));
        },
        LedgerCall::Burn { from, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &from)?;
            let old_bal = load_bal_must_exist(&ns_key, read_set)?;
            ensure!(old_bal >= amount, "insufficient balance");
            
            let new_bal = old_bal - amount ;
            write_set.insert(ns_key, Some(u128_to_be_vec(new_bal)));
        },
        LedgerCall::Transfer { from, to, amount} => {
            let from_key = address_to_entry_key(chain_id, &from)?;
            let to_key = address_to_entry_key(chain_id, &to)?;
            let from_old_bal = load_bal_must_exist(&from_key, read_set)?;
            let to_old_bal = load_bal_or_zero(&to_key, read_set);
            
            ensure!(from_old_bal >= amount, "insufficient balance");
            
            let from_new_bal = from_old_bal - amount;
            let to_new_bal = to_old_bal + amount;

            write_set.insert(from_key, Some(u128_to_be_vec(from_new_bal)));
            write_set.insert(to_key, Some(u128_to_be_vec(to_new_bal)));
        },
        LedgerCall::QueryBalance { addr} => {
            let ns_key = address_to_entry_key(chain_id, &addr)?;
            let old_val = load_bal_must_exist(&ns_key, read_set)?;
            answer = u128_to_be_vec(old_val);
        },
    }
    let ctr_outcome = CtrOutcome {
        write_set,
        answer,
    };
    Ok(ctr_outcome)
}

fn load_bal_or_zero(key: &NamespaceKey, read_set: &ReadSet) -> u128 {
    match read_set.get(&key) {
        Some(Some(value)) => { u128_from_be_slice(&value) },
        Some(None) | None => { 0 }
    }
}

fn load_bal_must_exist(key: &NamespaceKey, read_set: &ReadSet) -> Result<u128> {
    let value = read_set.get(&key)
        .ok_or_else(|| anyhow!("invalid ns_key"))?
        .as_ref()
        .ok_or_else(|| anyhow!("value is None"))?;
    let bal = u128_from_be_slice(&value);
    Ok(bal)
}