use account::address::{UserAddress};
use serde::{Deserialize, Serialize};
use spec::chain::ChainId;
use apps::ctr_io::{AccessSet, NamespaceKey};
use crate::error::Result;

pub const SMALLBANK_APP_NAME: &str = "smallbank";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmallbankCall {
    CreateAccount { customer_id: String, initial_checking_balance: i64, initial_savings_balance: i64 },
    TransactSavings { customer_id: String, amount: i64, }, //  支票账户存款 account.CheckingBalance += amount
    DepositChecking { customer_id: String, amount: i64, }, // 储蓄账户变动 account.SavingsBalance += amount
    // 转账 sourceAccount.CheckingBalance -= amount destAccount.CheckingBalance += amount
    SendPayment { source_customer_id: String, dest_customer_id: String, amount: i64, },
    WriteCheck { customer_id: String, amount: i64, }, // 支票账户扣款 account.CheckingBalance -= amount
    // 储蓄转入他人支票账户 destAccount.CheckingBalance += sourceAccount.SavingsBalance sourceAccount.SavingsBalance = 0
    Amalgamate { source_customer_id: String, dest_customer_id: String, },
    Query { customer_id: String, }, // 查询账户 customer_id
}

impl SmallbankCall {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

pub fn address_to_smallbank_entry_key(chain_id: ChainId, addr: &str) -> Result<NamespaceKey> {
    let addr = UserAddress::parse_bech32m_with_id(chain_id, addr)?;

    Ok(NamespaceKey {
        ns: SMALLBANK_APP_NAME.to_string(),
        key: Vec::from(addr.to_bytes()),
    })
}

pub fn generate_access_set(chain_id: ChainId, input: Vec<u8>) -> Result<AccessSet> {
    let call = SmallbankCall::try_decode_bcs(&input)?;
    let mut access_set = AccessSet::new();

    match call {
        SmallbankCall::CreateAccount { customer_id, .. } => { 
            access_set.insert(address_to_smallbank_entry_key(chain_id, &customer_id)?);
        }
        
        SmallbankCall::TransactSavings { customer_id, .. } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &customer_id)?);
        }

        SmallbankCall::DepositChecking { customer_id, .. } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &customer_id)?);
        }

        SmallbankCall::SendPayment {
            source_customer_id,
            dest_customer_id,
            ..
        } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &source_customer_id)?);
            access_set.insert(address_to_smallbank_entry_key(chain_id, &dest_customer_id)?);
        }

        SmallbankCall::WriteCheck { customer_id, .. } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &customer_id)?);
        }

        SmallbankCall::Amalgamate {
            source_customer_id,
            dest_customer_id,
        } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &source_customer_id)?);
            access_set.insert(address_to_smallbank_entry_key(chain_id, &dest_customer_id)?);
        }

        SmallbankCall::Query { customer_id } => {
            access_set.insert(address_to_smallbank_entry_key(chain_id, &customer_id)?);
        }
    }

    Ok(access_set)
}