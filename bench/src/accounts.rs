use std::path::PathBuf;
use account::keypair::{AccountSigningKey, AccountSigningKeyBytes, Keypair};
use std::{fs::File, io::{Write, BufWriter}};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use account::address::UserAddress;
use apps::ctr_io::NamespaceKey;
use ledger::call::address_to_entry_key;
use platform::config::load_base_config;
use spec::chain::ChainId;

pub fn accounts_path() -> PathBuf {
    let bench_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = bench_dir.parent().expect("bench should be under workspace root");
    workspace_dir.join("front/fixtures/workload/smallbank_accounts")
}

pub fn load_accounts() -> Vec<AccountSigningKey> {
    let path = accounts_path();
    let f = File::open(&path).expect("open smallbank_accounts failed");
    let reader = BufReader::new(f);
    let mut accounts: Vec<AccountSigningKey> = Vec::new();
    for line in reader.lines() {
        let s = line.expect("read line failed");
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        // 每行 hex -> Vec<u8>
        let raw: Vec<u8> = hex::decode(s).expect("hex decode failed");
        let sk_bytes: AccountSigningKeyBytes = raw.try_into().expect("invalid key length");
        // bytes -> AccountSigningKey
        let sk = AccountSigningKey::from_bytes(&sk_bytes);
        accounts.push(sk);
    }
    accounts
}

pub fn save_accounts() {
    let base = load_base_config();
    let accounts_num = base.workload.accounts_num;
    let f = File::create(accounts_path()).expect("create accounts failed");
    let mut writer = BufWriter::new(f);
    let users = gen_accounts(accounts_num);
    for sk in users {
        let sk_bytes: AccountSigningKeyBytes = sk.to_bytes();
        writeln!(writer, "{}", hex::encode(sk_bytes)).expect("hex encode failed");
    }
}

pub fn accounts_to_keys(chain_id: ChainId, accounts: &Vec<AccountSigningKey>) -> Vec<NamespaceKey> {
    let mut ns_keys = Vec::with_capacity(accounts.len());
    for account in accounts {
        let vk = account.verifying_key();
        let user_addr_str = UserAddress::from_vk(chain_id, &vk).to_bech32m().expect("encode bech32m failed");
        ns_keys.push(address_to_entry_key(chain_id, &user_addr_str).expect("address to entry key failed"))
    }
    ns_keys
}

pub fn accounts_to_map(chain_id: ChainId, accounts: &Vec<AccountSigningKey>) -> HashMap<String, AccountSigningKey> {
    let mut accounts_map = HashMap::with_capacity(accounts.len());
    for sk in accounts {
        let vk = sk.verifying_key();
        let user_addr_str = UserAddress::from_vk(chain_id, &vk).to_bech32m().expect("encode bech32m failed");
        accounts_map.insert(user_addr_str, sk.clone());
    }
    accounts_map
}

pub fn gen_accounts(accounts_num: usize) -> Vec<AccountSigningKey> {
    let mut accounts: Vec<AccountSigningKey> = Vec::with_capacity(accounts_num);
    for _ in 0..accounts_num {
        let sk = Keypair::generate().signing_key;
        accounts.push(sk);
    }
    accounts
}