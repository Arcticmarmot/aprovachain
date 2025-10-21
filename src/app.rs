use chrono::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};
use crate::common::{calc_hash, mine};

pub const DIFFICULTY_PREFIX: &str = "dd";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chain {
    pub blocks: Vec<Block>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub pre_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64
}

impl Block {
    pub fn new(id: u64, pre_hash: String, data: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let (nonce, hash) = mine(id, timestamp, &pre_hash, &data);
        Self {
            id,
            hash,
            pre_hash,
            timestamp,
            data,
            nonce
        }
    }

    pub fn is_block_valid(&self, pre_block: &Block) -> bool {
        let computed_hash = calc_hash(self.id, self.timestamp, &self.pre_hash, &self.data, self.nonce);
        computed_hash.starts_with(DIFFICULTY_PREFIX) && computed_hash == self.hash
            && self.pre_hash == pre_block.hash && self.id == pre_block.id + 1
    }
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            blocks: vec![]
        }
    }

    pub fn genesis(&mut self) {
        if self.blocks.len() > 0 {
            error!("the genesis block is already existing");
            return;
        }
        let genesis_block = Block {
            id: 0,
            hash: "None".to_string(),
            pre_hash: "None".to_string(),
            timestamp: Utc::now().timestamp(),
            data: "".to_string(),
            nonce: 0
        };
        self.blocks.push(genesis_block);
    }

    pub fn add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("the chain is empty");
        if block.is_block_valid(&latest_block) {
            self.blocks.push(block);
        } else {
            error!("could not add block: invalid block - {:?}", block);
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        let blocks = &self.blocks;
        if blocks.len() == 0 {
            false
        } else {
            for i in 0..blocks.len() {
                if i == 0 {
                    continue;
                }
                let block = &blocks[i];
                let pre_block = &blocks[i - 1];
                if block.is_block_valid(pre_block) {
                    return false;
                }
            }
            true
        }
    }
}

#[test]
fn test_app() {
    let b1 = Block::new(1, "None".to_string(), "hello blockchain".to_string());
    let b2 = Block::new(2, b1.hash.clone(), "hello blockchain again".to_string());

    let mut chain = Chain::new();
    chain.genesis();
    chain.add_block(b1);
    chain.add_block(b2);
    println!("{:?}", chain.blocks.last().unwrap());
}











