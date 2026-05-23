#![no_main]
#![no_std]

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;

use anyhow::{anyhow, Result};
use apps::ctr_io::{
    find_entry, CtrInput, CtrOutcome, CtrOutput, CtrResult, NamespaceKey, ReadSet, WriteSet,
};
use primitives::hash::sha256;
use risc0_zkvm::guest::env;
use smallbank::call::{address_to_entry_key, SmallbankCall};

risc0_zkvm::guest::entry!(main);

fn main() {
    let ctr_input_bytes: Vec<u8> = env::read();
    let input_hash = sha256(&ctr_input_bytes);

    let ctr_output: CtrOutput = match CtrInput::try_decode_bcs(&ctr_input_bytes) {
        Ok(ctr_input) => {
            let ctr_result = match handle_smallbank_call(&ctr_input) {
                Ok(ctr_outcome) => CtrResult::Ok {
                    outcome: ctr_outcome,
                },
                Err(err) => CtrResult::Err {
                    message: format!("{:#}", err),
                },
            };

            CtrOutput {
                input_hash,
                read_set: ctr_input.read_set,
                ctr_result,
            }
        }

        Err(err) => CtrOutput {
            input_hash,
            read_set: ReadSet::new(),
            ctr_result: CtrResult::Err {
                message: format!("{:#}", err),
            },
        },
    };

    env::commit(&ctr_output.encode_bcs());
}

fn handle_smallbank_call(ctr_input: &CtrInput) -> Result<CtrOutcome> {
    let chain_id = ctr_input.chain_id;
    let input = &ctr_input.input;
    let read_set = &ctr_input.read_set;

    let call = SmallbankCall::try_decode_bcs(input)?;

    let mut write_set = WriteSet::new();
    let mut answer: Vec<u8> = Vec::new();

    match call {
        SmallbankCall::CreateAccount {
            customer_id,
            initial_checking_balance,
            initial_savings_balance
        } => {
            let account_key = address_to_entry_key(chain_id, &customer_id)?;

            let account = SmallbankAccount {savings_balance: initial_savings_balance, checking_balance: initial_checking_balance};

            write_set.push((account_key, Some(account.encode())));
        }
        SmallbankCall::TransactSavings {
            customer_id,
            amount,
        } => {
            let account_key = address_to_entry_key(chain_id, &customer_id)?;
            let mut account = load_account_must_exist(&account_key, read_set)?;

            account.savings_balance += amount;

            write_set.push((account_key, Some(account.encode())));
        }

        SmallbankCall::DepositChecking {
            customer_id,
            amount,
        } => {
            let account_key = address_to_entry_key(chain_id, &customer_id)?;
            let mut account = load_account_must_exist(&account_key, read_set)?;

            account.checking_balance += amount;

            write_set.push((account_key, Some(account.encode())));
        }

        SmallbankCall::SendPayment {
            source_customer_id,
            dest_customer_id,
            amount,
        } => {
            let source_key = address_to_entry_key(chain_id, &source_customer_id)?;
            let dest_key = address_to_entry_key(chain_id, &dest_customer_id)?;

            let mut source_account = load_account_must_exist(&source_key, read_set)?;
            let mut dest_account = load_account_must_exist(&dest_key, read_set)?;

            source_account.checking_balance -= amount;
            dest_account.checking_balance += amount;

            write_set.push((source_key, Some(source_account.encode())));
            write_set.push((dest_key, Some(dest_account.encode())));
        }

        SmallbankCall::WriteCheck {
            customer_id,
            amount,
        } => {
            let account_key = address_to_entry_key(chain_id, &customer_id)?;
            let mut account = load_account_must_exist(&account_key, read_set)?;

            account.checking_balance -= amount;

            write_set.push((account_key, Some(account.encode())));
        }

        SmallbankCall::Amalgamate {
            source_customer_id,
            dest_customer_id,
        } => {
            let source_key = address_to_entry_key(chain_id, &source_customer_id)?;
            let dest_key = address_to_entry_key(chain_id, &dest_customer_id)?;

            let mut source_account = load_account_must_exist(&source_key, read_set)?;
            let mut dest_account = load_account_must_exist(&dest_key, read_set)?;

            dest_account.checking_balance += source_account.savings_balance;
            source_account.savings_balance = 0;

            write_set.push((source_key, Some(source_account.encode())));
            write_set.push((dest_key, Some(dest_account.encode())));
        }

        SmallbankCall::Query { customer_id } => {
            let account_key = address_to_entry_key(chain_id, &customer_id)?;
            let account = load_account_must_exist(&account_key, read_set)?;

            answer = account.encode();
        }
    }

    Ok(CtrOutcome { write_set, answer })
}

#[derive(Debug, Clone, Copy)]
struct SmallbankAccount {
    savings_balance: i64,
    checking_balance: i64,
}

impl SmallbankAccount {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.savings_balance.to_be_bytes());
        bytes.extend_from_slice(&self.checking_balance.to_be_bytes());
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            return Err(anyhow!("invalid smallbank account length"));
        }

        let savings_balance = i64_from_be_slice(&bytes[0..8]);
        let checking_balance = i64_from_be_slice(&bytes[8..16]);

        Ok(Self {
            savings_balance,
            checking_balance,
        })
    }
}

fn load_account_must_exist(key: &NamespaceKey, read_set: &ReadSet) -> Result<SmallbankAccount> {
    let value = find_entry(key, read_set)
        .ok_or_else(|| anyhow!("invalid ns_key"))?
        .as_ref()
        .ok_or_else(|| anyhow!("account value is None"))?;

    SmallbankAccount::decode(value)
}

fn i64_from_be_slice(bytes: &[u8]) -> i64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    i64::from_be_bytes(arr)
}