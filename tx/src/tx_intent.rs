use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use account::address::{ChainAddrBytes, UserAddress};
use account::keypair::{AccountVerifyingKey, AccountVerifyingKeyBytes};
use spec::chain::{ChainId};
use primitives::rand::random_u128;
use primitives::clock::unix_time_millis;
use primitives::hash::{sha256, Hash32};
use crate::error::TxError;
use crate::error::Result;
use crate::tx_id::TxIntentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxPayload {
    Exec {
        ctr_addr: ChainAddrBytes,
        input: Vec<u8>
    },
    Deploy {
        image_id: Digest,
        elf: Vec<u8>,
        elf_hash: Hash32
    },
    // TODO: 更新合约
    // Update {
    //     ctr_addr: ChainAddrBytes,
    //     image_id: Digest,
    //     elf: Vec<u8>,
    //     elf_hash: Hash32
    // }
}

#[derive(Debug)]
pub struct TxIntent {
    pub chain_id: ChainId,
    pub nonce: u128,
    pub address: UserAddress,
    pub verifying_key: AccountVerifyingKey,
    pub timestamp: u128,
    pub payload: TxPayload,
}

impl TryFrom<TxIntentWire> for TxIntent {
    type Error = TxError;

    fn try_from(wire: TxIntentWire) -> Result<Self> {
        let chain_id = ChainId(wire.chain_id);
        let address = UserAddress::from_bytes(wire.address);
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        Ok(Self {
            chain_id,
            nonce: wire.nonce,
            address,
            verifying_key,
            timestamp: wire.timestamp,
            payload: wire.payload
        })
    }
}


impl TxIntent {
    pub fn create(chain_id: ChainId, addr: UserAddress, vk: AccountVerifyingKey, payload: TxPayload) -> Result<Self> {
        let nonce = random_u128()?;
        let timestamp = unix_time_millis()?;
        Ok(Self {
            chain_id,
            nonce,
            timestamp,
            address: addr,
            verifying_key: vk,
            payload,
        })
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxIntentWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxIntentId {
        TxIntentId::new(sha256(self.to_canonical_bytes()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIntentWire {
    pub chain_id: u64,
    pub nonce: u128,
    pub address: ChainAddrBytes,
    pub verifying_key: AccountVerifyingKeyBytes,
    pub timestamp: u128,
    pub payload: TxPayload
}


impl TxIntentWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    /// NOTE: 输入为不可信的 BCS 编码字节切片；仅尝试解码
    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

impl From<&TxIntent> for TxIntentWire {
    fn from(intent: &TxIntent) -> Self {
        Self {
            chain_id: intent.chain_id.0,
            nonce: intent.nonce,
            address: intent.address.to_bytes(),
            verifying_key: intent.verifying_key.to_bytes(),
            timestamp: intent.timestamp,
            payload: intent.payload.clone()
        }
    }
}
