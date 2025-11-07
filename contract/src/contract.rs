use risc0_zkvm::Digest;
use account::address::{AddressBytes, ContractAddress};
use primitives::hash::Hash32;
use serde::{Deserialize, Serialize};
use account::keypair::AccountVerifyingKey;
use chain::spec::ChainId;

#[derive(Debug)]
pub struct Contract {
    pub addr: ContractAddress,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ContractWire {
    pub addr: AddressBytes,
    pub elf_hash: Hash32,
    pub image_id: Digest,
    pub salt: u128
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