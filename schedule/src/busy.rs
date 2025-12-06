use std::collections::BTreeMap;
use account::keypair::AccountVerifyingKey;


pub struct BusyFactor {

}

pub type BusyFactorTable = BTreeMap<AccountVerifyingKey, BusyFactor>;