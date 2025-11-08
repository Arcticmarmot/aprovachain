use risc0_zkvm::Digest;
use account::address::{Address, AddressBytes, ContractAddress};
use primitives::hash::Hash32;
use serde::{Deserialize, Serialize};
use account::keypair::AccountVerifyingKey;
use chain::spec::ChainId;
use crate::error::{ContractError,Result};
#[derive(Debug)]
pub struct Contract {
    pub addr: ContractAddress,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
}

impl Contract {
    pub fn create(chain_id: ChainId, elf_hash: Hash32, image_id: Digest,
                  vk: &AccountVerifyingKey, nonce: u128) -> Self {
        Self {
            addr: ContractAddress::create(chain_id, vk, nonce),
            elf_hash,
            image_id,
            salt: nonce
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let contract_wire = ContractWire::from(self);
        bcs::to_bytes(&contract_wire).expect("BCS should be infallible by design")
    }
}

impl TryFrom<ContractWire> for Contract {
    type Error = ContractError;

    fn try_from(wire: ContractWire) -> Result<Self> {
        let chain_id = ChainId(wire.chain_id);
        let ctr_addr = ContractAddress::create_from_bytes(chain_id, wire.addr);
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
    pub chain_id: u64,
    pub addr: AddressBytes,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
}

impl ContractWire {
    pub fn to_bcs_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn from_bcs_bytes(b: &[u8]) -> Self {
        bcs::from_bytes(b).expect("BCS should be infallible by design")
    }
}

impl From<&Contract> for ContractWire {
    fn from(ctr: &Contract) -> Self {
        Self {
            chain_id: ctr.addr.chain_id.0,
            addr: ctr.addr.to_bytes(),
            elf_hash: ctr.elf_hash,
            image_id: ctr.image_id,
            salt: ctr.salt
        }
    }
}