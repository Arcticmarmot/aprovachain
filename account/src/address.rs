//! Address protocol implementation

use core::iter;
use std::net::IpAddr;
use std::ops::Deref;
use rand::{CryptoRng, RngCore};
use serde::{Serialize, Deserialize};
use ed25519_dalek::Signer;

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
}

impl Address {

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











