use account::address::AccountPrefix;

pub const MAINNET_ID: u64 = 1000;
pub const TESTNET_ID: u64 = 2000;

pub enum Network {
    Mainnet,
    Testnet,
    Unknown(u64)
}

impl Network {
    pub fn from_chain_id(id: u64) -> Self {
        match id {
            MAINNET_ID => Network::Mainnet,
            TESTNET_ID => Network::Testnet,
            n => Network::Unknown(n)
        }
    }
}
pub trait ChainMetadata {
    const CHAIN_ID: u64;
    const HRP: &'static str;
}

pub struct Mainnet;

impl ChainMetadata for Mainnet {
    const CHAIN_ID: u64 = 1000;
    const HRP: &'static str = "aprova";
}

pub struct Testnet;

impl ChainMetadata for Testnet {
    const CHAIN_ID: u64 = 2000;
    const HRP: &'static str = "test";
}
