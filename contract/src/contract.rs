use std::fmt::{Debug};
use risc0_zkvm::Digest;
use account::address::{ChainAddrBytes, ContractAddress};
use primitives::hash::Hash32;
use serde::{Deserialize, Serialize};
use account::keypair::AccountVerifyingKey;
use spec::chain::ChainId;
use crate::error::{ContractError,Result};

#[derive(Debug, Clone)]
pub struct Contract {
    pub addr: ContractAddress,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
}

impl Contract {
    pub fn create(chain_id: ChainId, elf_hash: &Hash32, image_id: &Digest,
                  vk: &AccountVerifyingKey, nonce: u128) -> Self {
        Self {
            addr: ContractAddress::create(chain_id, vk, nonce),
            elf_hash: elf_hash.clone(),
            image_id: image_id.clone(),
            salt: nonce
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        ContractWire::from(self).decode_bcs()
    }
}

impl TryFrom<ContractWire> for Contract {
    type Error = ContractError;

    fn try_from(wire: ContractWire) -> Result<Self> {
        let ctr_addr = ContractAddress::from_bytes(wire.addr);
        Ok(Self {
            addr: ctr_addr,
            elf_hash: wire.elf_hash,
            image_id: wire.image_id,
            salt: wire.salt
        })
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ContractWire {
    pub addr: ChainAddrBytes,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
}

impl ContractWire {
    pub fn decode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_encode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

impl From<&Contract> for ContractWire {
    fn from(ctr: &Contract) -> Self {
        Self {
            addr: ctr.addr.to_bytes(),
            elf_hash: ctr.elf_hash,
            image_id: ctr.image_id,
            salt: ctr.salt
        }
    }
}