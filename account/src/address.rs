//! Address protocol implementation
//!
//! Address is a 64-byte string that allows anyone to deliver payment w/o exchanging Receivers.
//!
//! Sending funds this way incurs a bit of overhead: extra `data` entry in the transaction,
//! with 73 bytes of data. Last byte is used as a distinguisher to help identify the correct payload
//! out of multiple w/o computational overhead in case of multiple send-to-address outputs.
//!
//! Address consists of two 32-byte public keys (ristretto255 points): control key and encryption key.
//! Encryption key is used to encrypt the payment amount and arbitrary additional data,
//! while the control key allows spending the received funds.

use core::iter;
use std::net::IpAddr;
use std::ops::Deref;
use rand::{CryptoRng, RngCore};
use serde::{Serialize, Deserialize};
use merlin::Transcript;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::{IsIdentity, VartimeMultiscalarMul};
use bech32::{self, FromBase32, ToBase32};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddressLabel {
    inner: String
}

impl AddressLabel {
    pub fn new(label: String) -> Option<Self> {
        if let Ok(data) = bech32::encode(&label, [0x42u8; 1].to_base32()){
            println!("{}", data);
            Some(Self { inner: label })
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Deref for AddressLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Address {
    label: AddressLabel,
    control_key: CompressedRistretto,
    encryption_key: CompressedRistretto,
    encryption_key_decompressed: RistrettoPoint,
}

impl Address {
    pub(crate) fn new(
        label: AddressLabel,
        control_key: CompressedRistretto,
        encryption_key: RistrettoPoint
    ) -> Self {
        Self {
            label,
            control_key,
            encryption_key: encryption_key.compress(),
            encryption_key_decompressed: encryption_key
        }
    }

    pub fn label(&self) -> &AddressLabel {
        &self.label
    }

    pub fn control_key(&self) -> &CompressedRistretto {
        &self.control_key
    }

    pub fn to_string(&self) -> String {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.control_key.as_bytes());
        bytes.extend_from_slice(&self.encryption_key.as_bytes());
        bech32::encode(&self.label, bytes.to_base32())
            .expect("Label should be 1 to 83 characters long, printable ASCII, w/o mixing case.")
    }

    pub fn from_string(string: &str) -> Option<Self> {
        let (label, data) = bech32::decode(string).ok()?;
        let buf = Vec::<u8>::from_base32(&data).ok()?;
        if buf.len() != 64 {
            return None
        }
        let enckey = CompressedRistretto::from_slice(&buf[32..64]).decompress()?;
        Some(Address{
            label: AddressLabel { inner: label },
            control_key: CompressedRistretto::from_slice(&buf[0..32]),
            encryption_key: enckey.compress(),
            encryption_key_decompressed: enckey
        })
    }

}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_label() {
        assert_eq!(AddressLabel::new("".to_string()), None);
        AddressLabel::new("test".to_string());
    }
}











