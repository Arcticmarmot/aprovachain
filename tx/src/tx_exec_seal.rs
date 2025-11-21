use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey, AccountVerifyingKey, AccountVerifyingKeyBytes};
use primitives::hash::sha256;
use crate::error::TxError;
use crate::tx_exec::{TxExec, TxExecWire};
use crate::tx_id::TxExecSealId;
use crate::error::Result;


#[derive(Debug, Clone)]
pub struct TxExecSeal {
    pub tx_id: TxExecSealId,
    pub exec: TxExec,
    pub verifying_key: AccountVerifyingKey,
    pub signature: AccountSignature
}

impl PartialEq<Self> for TxExecSeal {
    fn eq(&self, other: &Self) -> bool {
        self.tx_id == other.tx_id
    }
}

impl Eq for TxExecSeal { }

impl Hash for TxExecSeal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_canonical_bytes());
    }
}

impl TryFrom<TxExecSealWire> for TxExecSeal {
    type Error = TxError;
    fn try_from(wire: TxExecSealWire) -> crate::error::Result<Self> {
        let exec = TxExec::try_from(wire.exec)?;
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        let signature = AccountSignature::from_bytes(&wire.signature);
        let tx_id = Self::compute_tx_id(&exec, &verifying_key, &signature);
        Ok(Self {
            tx_id,
            exec,
            verifying_key,
            signature
        })
    }
}

impl TxExecSeal {
    pub fn create(exec: TxExec, sk: AccountSigningKey) -> Self {
        let tx_exec_id = exec.tx_id();
        let vk = sk.verifying_key();
        let sig = sk.sign(&tx_exec_id.0);
        let tx_id = Self::compute_tx_id(&exec, &vk, &sig);
        Self {
            tx_id,
            exec,
            verifying_key: vk,
            signature: sig
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxExecSealWire::from(self).encode_bcs()
    }

    pub fn compute_tx_id(exec: &TxExec, vk: &AccountVerifyingKey, sig: &AccountSignature) -> TxExecSealId {
        let wire = TxExecSealWire {
            exec: exec.into(),
            verifying_key: vk.to_bytes(),
            signature: sig.to_bytes()
        };
        TxExecSealId::new(sha256(wire.encode_bcs()))
    }
    
    pub fn tx_id(&self) -> TxExecSealId {
        TxExecSealId::new(sha256(&self.to_canonical_bytes()))
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxExecSealWire {
    pub exec: TxExecWire,
    pub verifying_key: AccountVerifyingKeyBytes,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl From<&TxExecSeal> for TxExecSealWire {
    fn from(seal: &TxExecSeal) -> Self {
        let exec = &seal.exec;
        Self {
            exec: exec.into(),
            verifying_key: seal.verifying_key.to_bytes(),
            signature: seal.signature.to_bytes()
        }
    }
}

impl TxExecSealWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}


