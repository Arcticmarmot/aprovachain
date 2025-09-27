use crate::app::{Block, Chain};

mod p2p;
mod app;
mod common;

fn main() {
    let b1 = Block::new(1, "None".to_string(), "hello blockchain".to_string());
    let b2 = Block::new(2, b1.hash.clone(), "hello blockchain again".to_string());

    let mut chain = Chain::new();
    chain.genesis();
    chain.add_block(b1);
    chain.add_block(b2);
    println!("{:?}", chain.blocks.last().unwrap());

}
