use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey, AccountVerifyingKey, AccountVerifyingKeyBytes};
use primitives::hash::sha256;
use crate::error::TxError;
use crate::outcome::{TxOutcome, TxOutcomeWire};
use crate::id::TxAttestationId;
use crate::error::Result;


#[derive(Debug, Clone)]
pub struct TxAttestation {
    pub tx_id: TxAttestationId,
    pub outcome: TxOutcome,
    pub verifying_key: AccountVerifyingKey,
    pub signature: AccountSignature
}

impl PartialEq<Self> for TxAttestation {
    fn eq(&self, other: &Self) -> bool {
        self.tx_id == other.tx_id
    }
}

impl Eq for TxAttestation { }

impl Hash for TxAttestation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_canonical_bytes());
    }
}

impl TryFrom<TxAttestationWire> for TxAttestation {
    type Error = TxError;
    fn try_from(wire: TxAttestationWire) -> Result<Self> {
        let outcome = TxOutcome::try_from(wire.outcome)?;
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        let signature = AccountSignature::from_bytes(&wire.signature);
        let tx_id = Self::compute_tx_id(&outcome, &verifying_key, &signature);
        Ok(Self {
            tx_id,
            outcome,
            verifying_key,
            signature
        })
    }
}

impl TxAttestation {
    pub fn create(outcome: TxOutcome, sk: AccountSigningKey) -> Self {
        let tx_outcome_id = outcome.tx_id();
        let vk = sk.verifying_key();
        let sig = sk.sign(&tx_outcome_id.0);
        let tx_id = Self::compute_tx_id(&outcome, &vk, &sig);
        Self {
            tx_id,
            outcome,
            verifying_key: vk,
            signature: sig
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxAttestationWire::from(self).encode_bcs()
    }

    pub fn compute_tx_id(outcome: &TxOutcome, vk: &AccountVerifyingKey, sig: &AccountSignature) -> TxAttestationId {
        let wire = TxAttestationWire {
            outcome: outcome.into(),
            verifying_key: vk.to_bytes(),
            signature: sig.to_bytes()
        };
        TxAttestationId::new(sha256(wire.encode_bcs()))
    }
    
    pub fn tx_id(&self) -> TxAttestationId {
        TxAttestationId::new(sha256(&self.to_canonical_bytes()))
    }
    
    pub fn self_verify(&self) -> Result<()> {
        Ok(self.verifying_key.verify(&self.outcome.tx_id().0, &self.signature)?)
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxAttestationWire {
    pub outcome: TxOutcomeWire,
    pub verifying_key: AccountVerifyingKeyBytes,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl From<&TxAttestation> for TxAttestationWire {
    fn from(seal: &TxAttestation) -> Self {
        let outcome = &seal.outcome;
        Self {
            outcome: outcome.into(),
            verifying_key: seal.verifying_key.to_bytes(),
            signature: seal.signature.to_bytes()
        }
    }
}

impl TxAttestationWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}


