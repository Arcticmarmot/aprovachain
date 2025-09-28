use chrono::Utc;
use crate::app::{Block, Chain};
use crate::p2p::discovery;

mod p2p;
mod app;
mod common;
#[tokio::main]
async fn main() {
    // let b1 = Block::new(1, "None".to_string(), "hello blockchain".to_string());
    // let b2 = Block::new(2, b1.hash.clone(), "hello blockchain again".to_string());
    //
    // let mut chain = Chain::new();
    // chain.genesis();
    // chain.add_block(b1);
    // chain.add_block(b2);
    // println!("{:?}", chain.blocks.last().unwrap());
    let timestamp = Utc::now().timestamp().to_string();
    discovery(timestamp).await;
}
