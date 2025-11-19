use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey, AccountVerifyingKey, AccountVerifyingKeyBytes};
use crate::error::TxError;
use crate::tx_exec::{TxExec, TxExecWire};

pub struct TxExecSeal {
    pub exec: TxExec,
    pub verifying_key: AccountVerifyingKey,
    pub signature: AccountSignature
}

impl TryFrom<TxExecSealWire> for TxExecSeal {
    type Error = TxError;
    fn try_from(wire: TxExecSealWire) -> crate::error::Result<Self> {
        let exec = TxExec::try_from(wire.exec)?;
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        let signature = AccountSignature::from_bytes(&wire.signature);
        Ok(Self {
            exec,
            verifying_key,
            signature
        })
    }
}

impl TxExecSeal {
    pub fn create(exec: TxExec, vk: AccountVerifyingKey, sk: AccountSigningKey) -> Self {
        let tx_exec_id = exec.tx_exec_id();
        let sig = sk.sign(&tx_exec_id.0);
        Self {
            exec,
            verifying_key: vk,
            signature: sig
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxExecSealWire::from(self).encode_bcs()
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
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

    pub fn try_decode_bcs(bytes: &[u8]) -> crate::error::Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}


