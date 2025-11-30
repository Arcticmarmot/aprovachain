#![no_main]
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;
use apps::ctr_io::*;
use ledger::call::*;
use anyhow::{anyhow, ensure, Result};
use primitives::hash::sha256;
use primitives::trans::{u128_from_be_slice, u128_to_be_vec};

risc0_zkvm::guest::entry!(main);
fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let input_hash = sha256(&ctr_input_bytes);
    let ctr_result: CtrResult;
    match handle_ledger_call(ctr_input_bytes) {
        Ok(ctr_outcome) => {
            ctr_result = CtrResult::Ok {
                outcome: ctr_outcome
            };
        },
        Err(err) => {
            ctr_result = CtrResult::Err {
                message: format!("{:#}", err)
            };
        }
    }
    let ctr_output = CtrOutput {
        input_hash,
        ctr_result
    };
    env::commit(&ctr_output.encode_bcs())
}

fn handle_ledger_call(ctr_input_bytes: Vec<u8>) -> Result<CtrOutcome> {
    let ctr_input = CtrInput::try_decode_bcs(&ctr_input_bytes)?;
    let chain_id = ctr_input.chain_id;
    let context = ctr_input.context;
    let input = ctr_input.input;
    let call = LedgerCall::try_decode_bcs(&input)?;
    let mut write_set = WriteSet::new();
    let mut answer: Vec<u8> = Vec::new();
    match call {
        LedgerCall::Mint { to, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &to)?;
            let (old_ver, old_bal) = load_snap_or_zero(&context, &ns_key)?;
            let new_bal = old_bal + amount;
            let new_snap = build_snapshot(Some(old_ver), new_bal);
            write_set.insert(ns_key, Some(new_snap));
        },
        LedgerCall::Burn { from, amount } => {
            // 读取用户地址账户数据
            let ns_key = address_to_entry_key(chain_id, &from)?;
            let (old_ver, old_bal) = load_snap_must_exist(&context, &ns_key)?;
            ensure!(old_bal >= amount,  
                "insufficient balance for burn: have {}, burn {}", old_bal, amount);
            
            let new_bal = old_bal - amount ;
            let new_snap = build_snapshot(Some(old_ver), new_bal);
            write_set.insert(ns_key, Some(new_snap));
        },
        LedgerCall::Transfer { from, to, amount} => {
            let from_key = address_to_entry_key(chain_id, &from)?;
            let to_key = address_to_entry_key(chain_id, &to)?;
            let (from_old_ver, from_old_bal) = load_snap_must_exist(&context, &from_key)?;
            let (to_old_ver, to_old_bal) = load_snap_or_zero(&context, &to_key)?;
            
            ensure!(from_old_bal >= amount, 
                "insufficient balance for burn: have {}, burn {}", from_old_bal, amount);
            
            let from_new_bal = from_old_bal - amount;
            let to_new_bal = to_old_bal + amount;

            let from_new_snap = build_snapshot(Some(from_old_ver), from_new_bal);
            
            let to_new_snap = build_snapshot(Some(to_old_ver), to_new_bal);

            write_set.insert(from_key, Some(from_new_snap));
            write_set.insert(to_key, Some(to_new_snap));
        },
        LedgerCall::QueryBalance { addr} => {
            let ns_key = address_to_entry_key(chain_id, &addr)?;
            let (_, old_val) = load_snap_must_exist(&context, &ns_key)?;
            answer = u128_to_be_vec(old_val);
        },
    }
    let ctr_outcome = CtrOutcome {
        effects: CtrEffects {
            read_set: context.read_set,
            write_set
        },
        answer,
    };
    Ok(ctr_outcome)
}

fn load_snap_or_zero(context: &EnvContext, key: &NamespaceKey) -> Result<(u128, u128)> {
    match context.read_set.get(&key) {
        Some(Some(snap)) => {
            let old_ver = snap.version;
            let old_bal = u128_from_be_slice(&snap.value);
            Ok((old_ver, old_bal))
        },
        Some(None) | None => {
            Ok((0, 0))
        }
    }
}

fn load_snap_must_exist(context: &EnvContext, key: &NamespaceKey) -> Result<(u128, u128)> {
    let snap = context
        .read_set
        .get(&key)
        .ok_or_else(|| anyhow!("invalid namespace key: {:?}", key))?
        .as_ref()
        .ok_or_else(|| anyhow!("snapshot is None for burn: {:?}", key))?;
    let old_ver = snap.version;
    let old_bal = u128_from_be_slice(&snap.value);
    Ok((old_ver, old_bal))
}

fn build_snapshot(old_ver_opt: Option<u128>, new_bal: u128) -> ValueSnapshot {
    let new_ver = match old_ver_opt {
        Some(old_ver) => old_ver + 1,
        None => 0
    };
    ValueSnapshot {
        version: new_ver,
        value: u128_to_be_vec(new_bal)
    }
}