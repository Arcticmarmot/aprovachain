use serde::{Deserialize, Serialize};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey, AccountVerifyingKey, AccountVerifyingKeyBytes};
use primitives::hash::{sha256};
use crate::error::TxError;
use serde_with::{serde_as, Bytes};
use account::address::{ChainAddrBytes, UserAddress};
use crate::intent::{TxIntent, TxIntentWire};
use crate::error::Result;
use crate::id::TxEnvelopeId;

#[derive(Debug, Clone)]
pub struct TxEnvelope {
    pub intent: TxIntent,
    pub verifying_key: AccountVerifyingKey,
    pub address: UserAddress,
    pub signature: AccountSignature,
}

impl TxEnvelope {
    pub fn create(intent: TxIntent, sk: AccountSigningKey) -> Self {
        // 使用私钥对 TxIntent 计算出的 tx_intent_id 进行签名
        let tx_intent_id = intent.tx_id();
        let verifying_key = sk.verifying_key();
        let address = UserAddress::from_vk(intent.chain_id, &verifying_key);
        let signature: AccountSignature = sk.sign(&tx_intent_id.0);
        Self {
            intent,
            verifying_key,
            address,
            signature
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxEnvelopeWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxEnvelopeId {
        TxEnvelopeId::new(sha256(self.to_canonical_bytes()))
    }

    pub fn self_verify(&self) -> Result<()> {
        Ok(self.verifying_key.verify(&self.intent.tx_id().0, &self.signature)?)
    }
}

impl TryFrom<TxEnvelopeWire> for TxEnvelope {
    type Error = TxError;

    fn try_from(wire: TxEnvelopeWire) -> Result<Self> {
        let intent = TxIntent::try_from(wire.intent)?;
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        let address = UserAddress::from_bytes(wire.address);
        let signature = AccountSignature::from_bytes(&wire.signature);
        Ok(Self {
            intent,
            verifying_key,
            address,
            signature
        })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEnvelopeWire {
    pub intent: TxIntentWire,
    pub verifying_key: AccountVerifyingKeyBytes,
    pub address: ChainAddrBytes,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl From<&TxEnvelope> for TxEnvelopeWire {
    fn from(envelope: &TxEnvelope) -> Self {
        let intent = &envelope.intent;
        Self {
            intent: intent.into(),
            verifying_key: envelope.verifying_key.to_bytes(),
            address: envelope.address.to_bytes(),
            signature: envelope.signature.to_bytes()
        }
    }
}


impl TxEnvelopeWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

