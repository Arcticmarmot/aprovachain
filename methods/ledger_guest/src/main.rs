#![no_main]
#![no_std]
extern crate alloc;

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
    // TODO: 错误返回给调用者
    let ctr_output_bytes = handle_ledger_call(ctr_input_bytes).unwrap();
    env::commit(&ctr_output_bytes)
}

fn handle_ledger_call(ctr_input_bytes: Vec<u8>) -> Result<CtrOutput> {
    let ctr_input = CtrInput::try_decode_bcs(&ctr_input_bytes)?;
    let chain_id = ctr_input.chain_id;
    let context = ctr_input.context;
    let input = ctr_input.input;
    let call = LedgerCall::try_decode_bcs(&input)?;
    let mut write_set = WriteSet::new();
    let mut output: Vec<u8> = Vec::new();
    match call {
        LedgerCall::Mint { to, amount } => {
            // 读取用户地址账户数据
            let to_key = address_to_entry_key(chain_id, &to)?;
            let to_snap_opt = context.read_set.get(&to_key);
            
            let (old_version, old_bal) = match to_snap_opt {
                Some(Some(snap)) => {
                    (snap.version, u128_from_be_slice(&snap.value))
                }
                Some(None) | None => (0u128, 0u128),
            };
            let new_version = old_version + 1;
            let new_bal = old_bal + amount;

            let new_snap = ValueSnapShot {
                version: new_version,
                value: u128_to_be_vec(new_bal),
            };
            
            write_set.insert(to_key, Some(new_snap));
        },
        LedgerCall::Burn { from, amount } => {
            // 读取用户地址账户数据
            let from_key = address_to_entry_key(chain_id, &from)?;
            let snap = context
                .read_set
                .get(&from_key)
                .ok_or_else(|| anyhow!("invalid key for burn: {:?}", from_key))?
                .as_ref()
                .ok_or_else(|| anyhow!("snapshot is None for burn: {:?}", from_key))?;
            let new_version = snap.version + 1;
            let old_bal = u128_from_be_slice(&snap.value);
            ensure!(old_bal >= amount,  
                "insufficient balance for burn: have {}, burn {}", old_bal, amount);
            
            let new_value = old_bal - amount ;
            let new_snap = ValueSnapShot {
                version: new_version,
                value: u128_to_be_vec(new_value)
            };
            write_set.insert(from_key, Some(new_snap));
        },
        LedgerCall::Transfer { from, to, amount} => {
            let from_key = address_to_entry_key(chain_id, &from)?;
            let to_key = address_to_entry_key(chain_id, &to)?;
            let from_snap = context
                .read_set
                .get(&from_key)
                .ok_or_else(|| anyhow!("invalid key for burn: {:?}", from_key))?
                .as_ref()
                .ok_or_else(|| anyhow!("snapshot is None for burn: {:?}", from_key))?;
            
            let to_snap_opt = context.read_set.get(&to_key);

            let (to_old_version, to_old_bal) = match to_snap_opt {
                Some(Some(snap)) => {
                    let bal = u128_from_be_slice(&snap.value);
                    (snap.version, bal)
                }
                Some(None) | None => (0u128, 0u128),
            };
            
            let from_old_bal = u128_from_be_slice(&from_snap.value);
            
            ensure!(from_old_bal >= amount, 
                "insufficient balance for burn: have {}, burn {}", from_old_bal, amount);
            
            let from_new_version = from_snap.version + 1;
            let from_new_bal = from_old_bal - amount;
            let to_new_version = to_old_version + 1;
            let to_new_bal = to_old_bal + amount;

            let from_new_snap = ValueSnapShot {
                version: from_new_version,
                value: u128_to_be_vec(from_new_bal)
            };
            
            let to_new_snap = ValueSnapShot {
                version: to_new_version,
                value: u128_to_be_vec(to_new_bal),
            };

            write_set.insert(from_key, Some(from_new_snap));
            write_set.insert(to_key, Some(to_new_snap));
        },
        LedgerCall::QueryBalance { addr} => {
            let ns_key = address_to_entry_key(chain_id, &addr)?;
            let snap = context
                .read_set
                .get(&ns_key)
                .ok_or_else(|| anyhow!("invalid key for burn: {:?}", ns_key))?
                .as_ref()
                .ok_or_else(|| anyhow!("snapshot is None for burn: {:?}", ns_key))?;
            output = snap.value.clone()
        },
    }
    let ctr_output = CtrOutput {
        input_hash: sha256(ctr_input_bytes),
        read_set: context.read_set,
        write_set,
        output,
    };
    Ok(ctr_output)
}