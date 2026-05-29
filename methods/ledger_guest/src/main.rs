#![no_main]
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use apps::ctr_io::{CtrInput, CtrOutcome, CtrOutput, CtrResult, NamespaceKey, WriteSet, ReadSet, find_entry};
use ledger::call::{address_to_ledger_entry_key, LedgerCall};
use anyhow::{anyhow, ensure, Result};
use primitives::hash::{sha256};
use primitives::trans::{u64_from_be_slice, u64_to_be_vec};

risc0_zkvm::guest::entry!(main);
fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let input_hash = sha256(&ctr_input_bytes);
    let ctr_output: CtrOutput = match CtrInput::try_decode_bcs(&ctr_input_bytes) {
        Ok(ctr_input) => {
            let ctr_result = match handle_ledger_call(&ctr_input) {
                Ok(ctr_outcome) => {
                    CtrResult::Ok { outcome: ctr_outcome }
                },
                Err(err) => {
                    CtrResult::Err { message: format!("{:#}", err) }
                }
            };
            CtrOutput {
                input_hash,
                read_set: ctr_input.read_set,
                ctr_result
            }
        },
        Err(err) => {
            CtrOutput {
                input_hash,
                read_set: ReadSet::new(),
                ctr_result: CtrResult::Err {
                    message: format!("{:#}", err)
                }
            }
        }
    };
    env::commit(&ctr_output.encode_bcs());
}

fn handle_ledger_call(ctr_input: &CtrInput) -> Result<CtrOutcome> {
    let chain_id = ctr_input.chain_id;
    let input = &ctr_input.input;
    let read_set = &ctr_input.read_set;
    let call = LedgerCall::try_decode_bcs(input)?;
    let mut write_set = WriteSet::new();
    let mut answer: Vec<u8> = Vec::new();
    match call {
        LedgerCall::Mint { to, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_ledger_entry_key(chain_id, &to)?;
            let old_bal = load_bal_or_zero(&ns_key, read_set);
            let new_bal = old_bal + amount;
            write_set.push((ns_key, Some(u64_to_be_vec(new_bal))));
        },
        LedgerCall::Burn { from, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_ledger_entry_key(chain_id, &from)?;
            let old_bal = load_bal_must_exist(&ns_key, read_set)?;
            ensure!(old_bal >= amount, "insufficient balance");
            
            let new_bal = old_bal - amount ;
            write_set.push((ns_key, Some(u64_to_be_vec(new_bal))));
        },
        LedgerCall::Transfer { from, to, amount} => {
            let from_key = address_to_ledger_entry_key(chain_id, &from)?;
            let to_key = address_to_ledger_entry_key(chain_id, &to)?;
            let from_old_bal = load_bal_must_exist(&from_key, read_set)?;
            let to_old_bal = load_bal_or_zero(&to_key, read_set);
            
            ensure!(from_old_bal >= amount, "insufficient balance");
            
            let from_new_bal = from_old_bal - amount;
            let to_new_bal = to_old_bal + amount;

            write_set.push((from_key, Some(u64_to_be_vec(from_new_bal))));
            write_set.push((to_key, Some(u64_to_be_vec(to_new_bal))));
        },
        LedgerCall::QueryBalance { addr} => {
            let ns_key = address_to_ledger_entry_key(chain_id, &addr)?;
            let old_val = load_bal_must_exist(&ns_key, read_set)?;
            answer = u64_to_be_vec(old_val);
        },
    }
    let ctr_outcome = CtrOutcome {
        write_set,
        answer,
    };
    Ok(ctr_outcome)
}

fn load_bal_or_zero(key: &NamespaceKey, read_set: &ReadSet) -> u64 {
    match find_entry(key, read_set) {
        Some(Some(value)) => { u64_from_be_slice(value) },
        Some(None) | None => { 0 }
    }
}

fn load_bal_must_exist(key: &NamespaceKey, read_set: &ReadSet) -> Result<u64> {
    let value = find_entry(key, read_set)
        .ok_or_else(|| anyhow!("invalid ns_key"))?
        .as_ref()
        .ok_or_else(|| anyhow!("value is None"))?;
    let bal = u64_from_be_slice(value);
    Ok(bal)
}