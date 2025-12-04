#![no_main]
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use apps::ctr_io::{CtrInput, CtrOutcome, CtrOutput, CtrResult, EnvContext, NamespaceKey, WriteSet};
use ledger::call::{address_to_entry_key, LedgerCall};
use anyhow::{anyhow, ensure, Result};
use primitives::hash::sha256;
use primitives::trans::{u128_from_be_slice, u128_to_be_vec};
use spec::chain::ChainId;

risc0_zkvm::guest::entry!(main);
fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let input_hash = sha256(&ctr_input_bytes);

    let ctr_output: CtrOutput = match CtrInput::try_decode_bcs(&ctr_input_bytes) {
        Ok(ctr_input) => {
            let chain_id = ctr_input.chain_id;
            let context = ctr_input.context;
            let input = ctr_input.input;
            let ctr_result = match handle_ledger_call(chain_id, &context, &input) {
                Ok(ctr_outcome) => {
                    CtrResult::Ok { outcome: ctr_outcome }
                },
                Err(err) => {
                    CtrResult::Err { message: format!("{:#}", err) }
                }
            };
            CtrOutput {
                input_hash,
                context,
                ctr_result
            }
        },
        Err(err) => {
            CtrOutput {
                input_hash,
                context: EnvContext::new(),
                ctr_result: CtrResult::Err {
                    message: format!("{:#}", err)
                }
            }
        }
    };
    env::commit(&ctr_output.encode_bcs());
}

fn handle_ledger_call(chain_id: ChainId, context: &EnvContext, input: &Vec<u8>) -> Result<CtrOutcome> {
    let call = LedgerCall::try_decode_bcs(&input)?;
    let mut write_set = WriteSet::new();
    let mut answer: Vec<u8> = Vec::new();
    match call {
        LedgerCall::Mint { to, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &to)?;
            let old_bal = load_bal_or_zero(&context, &ns_key)?;
            let new_bal = old_bal + amount;
            write_set.insert(ns_key, Some(u128_to_be_vec(new_bal)));
        },
        LedgerCall::Burn { from, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &from)?;
            let old_bal = load_bal_must_exist(&context, &ns_key)?;
            ensure!(old_bal >= amount,  
                "insufficient balance for burn: have {}, burn {}", old_bal, amount);
            
            let new_bal = old_bal - amount ;
            write_set.insert(ns_key, Some(u128_to_be_vec(new_bal)));
        },
        LedgerCall::Transfer { from, to, amount} => {
            let from_key = address_to_entry_key(chain_id, &from)?;
            let to_key = address_to_entry_key(chain_id, &to)?;
            let from_old_bal = load_bal_must_exist(&context, &from_key)?;
            let to_old_bal = load_bal_or_zero(&context, &to_key)?;
            
            ensure!(from_old_bal >= amount, 
                "insufficient balance for burn: have {}, burn {}", from_old_bal, amount);
            
            let from_new_bal = from_old_bal - amount;
            let to_new_bal = to_old_bal + amount;

            write_set.insert(from_key, Some(u128_to_be_vec(from_new_bal)));
            write_set.insert(to_key, Some(u128_to_be_vec(to_new_bal)));
        },
        LedgerCall::QueryBalance { addr} => {
            let ns_key = address_to_entry_key(chain_id, &addr)?;
            let old_val = load_bal_must_exist(&context, &ns_key)?;
            answer = u128_to_be_vec(old_val);
        },
    }
    let ctr_outcome = CtrOutcome {
        write_set,
        answer,
    };
    Ok(ctr_outcome)
}

fn load_bal_or_zero(context: &EnvContext, key: &NamespaceKey) -> Result<u128> {
    match context.read_set.get(&key) {
        Some(Some(snap)) => {
            let bal = u128_from_be_slice(&snap.value);
            Ok(bal)
        },
        Some(None) | None => {
            Ok(0)
        }
    }
}

fn load_bal_must_exist(context: &EnvContext, key: &NamespaceKey) -> Result<u128> {
    let snap = context
        .read_set
        .get(&key)
        .ok_or_else(|| anyhow!("invalid namespace key: {:?}", key))?
        .as_ref()
        .ok_or_else(|| anyhow!("snapshot is None for burn: {:?}", key))?;
    let bal = u128_from_be_slice(&snap.value);
    Ok(bal)
}