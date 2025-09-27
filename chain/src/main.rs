use crate::app::Block;

mod p2p;
mod app;
mod common;

fn main() {
    let b = Block::new(1, "dd3f9b978558184603e6c4ed4195b35dbce9c643e156d330169df7ecc9c33adf".to_string(), "hello blockchain".to_string());
    println!("{:?}", b);
    let b = Block::new(2, b.hash, "hello blockchain again".to_string());
    println!("{:?}", b);
}
