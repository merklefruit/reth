//! Mock Types

use crate::{
    identifier::{SenderIdentifiers, TransactionId},
    pool::txpool::TxPool,
    traits::TransactionOrigin,
    PoolTransaction, TransactionOrdering, ValidPoolTransaction,
};
use paste::paste;
use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::Distribution,
};
use reth_primitives::{
    constants::MIN_PROTOCOL_BASE_FEE, hex, Address, FromRecoveredTransaction,
    IntoRecoveredTransaction, Signature, Transaction, TransactionKind, TransactionSigned,
    TransactionSignedEcRecovered, TxEip1559, TxHash, TxLegacy, TxType, H256, U128, U256,
};
use std::{ops::Range, sync::Arc, time::Instant};

pub(crate) type MockTxPool = TxPool<MockOrdering>;

pub type MockValidTx = ValidPoolTransaction<MockTransaction>;

#[cfg(feature = "optimism")]
use reth_primitives::DEPOSIT_TX_TYPE;

#[cfg(feature = "optimism")]
use reth_primitives::TxDeposit;

#[cfg(feature = "optimism")]
use reth_primitives::Bytes;

/// Create an empty `TxPool`
pub(crate) fn mock_tx_pool() -> MockTxPool {
    MockTxPool::new(Default::default(), Default::default())
}

/// Sets the value for the field
macro_rules! set_value {
    ($this:ident => $field:ident) => {
        let new_value = $field;
        match $this {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { ref mut $field, .. } => {
                *$field = new_value;
            }
            MockTransaction::Legacy { ref mut $field, .. } => {
                *$field = new_value;
            }
            MockTransaction::Eip1559 { ref mut $field, .. } => {
                *$field = new_value;
            }
        }
    };
}

/// Sets the value fo the field
macro_rules! get_value {
    ($this:ident => $field:ident) => {
        match $this {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { $field, .. } => $field,
            MockTransaction::Legacy { $field, .. } => $field,
            MockTransaction::Eip1559 { $field, .. } => $field,
        }
    };
}

// Generates all setters and getters
macro_rules! make_setters_getters {
    ($($name:ident => $t:ty);*) => {
  paste! {
        $(
            pub fn [<set_ $name>](&mut self, $name: $t) -> &mut Self {
                set_value!(self => $name);
                self
            }

            pub fn [<with_$name>](mut self, $name: $t) -> Self {
                set_value!(self => $name);
                self
            }

            pub fn [<get_$name>](&self) -> $t {
                get_value!(self => $name).clone()
            }

        )*

    }
    };
}

/// A Bare transaction type used for testing.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MockTransaction {
    Legacy {
        hash: H256,
        sender: Address,
        nonce: u64,
        gas_price: u128,
        gas_limit: u64,
        to: TransactionKind,
        value: U256,
    },
    Eip1559 {
        hash: H256,
        sender: Address,
        nonce: u64,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
        gas_limit: u64,
        to: TransactionKind,
        value: U256,
    },
    #[cfg(feature = "optimism")]
    DepositTx {
        hash: H256,
        sender: Address,
        nonce: u64,
        to: TransactionKind,
        mint: Option<u128>,
        gas_limit: u64,
        is_system_transaction: bool,
        input: Bytes,
        value: U256,
    },
}

// === impl MockTransaction ===

impl MockTransaction {
    make_setters_getters! {
        nonce => u64;
        hash => H256;
        sender => Address;
        gas_limit => u64;
        value => U256
    }

    /// Returns a new legacy transaction with random address and hash and empty values
    pub fn legacy() -> Self {
        MockTransaction::Legacy {
            hash: H256::random(),
            sender: Address::random(),
            nonce: 0,
            gas_price: 0,
            gas_limit: 0,
            to: TransactionKind::Call(Address::random()),
            value: Default::default(),
        }
    }

    /// Returns a new EIP1559 transaction with random address and hash and empty values
    pub fn eip1559() -> Self {
        MockTransaction::Eip1559 {
            hash: H256::random(),
            sender: Address::random(),
            nonce: 0,
            max_fee_per_gas: MIN_PROTOCOL_BASE_FEE as u128,
            max_priority_fee_per_gas: MIN_PROTOCOL_BASE_FEE as u128,
            gas_limit: 0,
            to: TransactionKind::Call(Address::random()),
            value: Default::default(),
        }
    }

    pub fn set_priority_fee(&mut self, val: u128) -> &mut Self {
        if let MockTransaction::Eip1559 { max_priority_fee_per_gas, .. } = self {
            *max_priority_fee_per_gas = val;
        }
        self
    }

    pub fn with_priority_fee(mut self, val: u128) -> Self {
        if let MockTransaction::Eip1559 { ref mut max_priority_fee_per_gas, .. } = self {
            *max_priority_fee_per_gas = val;
        }
        self
    }

    pub fn get_priority_fee(&self) -> Option<u128> {
        if let MockTransaction::Eip1559 { max_priority_fee_per_gas, .. } = self {
            Some(*max_priority_fee_per_gas)
        } else {
            None
        }
    }

    pub fn set_max_fee(&mut self, val: u128) -> &mut Self {
        if let MockTransaction::Eip1559 { max_fee_per_gas, .. } = self {
            *max_fee_per_gas = val;
        }
        self
    }

    pub fn with_max_fee(mut self, val: u128) -> Self {
        if let MockTransaction::Eip1559 { ref mut max_fee_per_gas, .. } = self {
            *max_fee_per_gas = val;
        }
        self
    }

    pub fn get_max_fee(&self) -> Option<u128> {
        if let MockTransaction::Eip1559 { max_fee_per_gas, .. } = self {
            Some(*max_fee_per_gas)
        } else {
            None
        }
    }

    pub fn set_gas_price(&mut self, val: u128) -> &mut Self {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => {}
            MockTransaction::Legacy { gas_price, .. } => {
                *gas_price = val;
            }
            MockTransaction::Eip1559 { max_fee_per_gas, max_priority_fee_per_gas, .. } => {
                *max_fee_per_gas = val;
                *max_priority_fee_per_gas = val;
            }
        }
        self
    }

    pub fn with_gas_price(mut self, val: u128) -> Self {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => {}
            MockTransaction::Legacy { ref mut gas_price, .. } => {
                *gas_price = val;
            }
            MockTransaction::Eip1559 {
                ref mut max_fee_per_gas,
                ref mut max_priority_fee_per_gas,
                ..
            } => {
                *max_fee_per_gas = val;
                *max_priority_fee_per_gas = val;
            }
        }
        self
    }

    pub fn get_gas_price(&self) -> u128 {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => 0,
            MockTransaction::Legacy { gas_price, .. } => *gas_price,
            MockTransaction::Eip1559 { max_fee_per_gas, .. } => *max_fee_per_gas,
        }
    }

    /// Returns a clone with a decreased nonce
    pub fn prev(&self) -> Self {
        let mut next = self.clone().with_hash(H256::random());
        next.with_nonce(self.get_nonce() - 1)
    }

    /// Returns a clone with an increased nonce
    pub fn next(&self) -> Self {
        let mut next = self.clone().with_hash(H256::random());
        next.with_nonce(self.get_nonce() + 1)
    }

    /// Returns a clone with an increased nonce
    pub fn skip(&self, skip: u64) -> Self {
        let mut next = self.clone().with_hash(H256::random());
        next.with_nonce(self.get_nonce() + skip + 1)
    }

    /// Returns a clone with incremented nonce
    pub fn inc_nonce(mut self) -> Self {
        let nonce = self.get_nonce() + 1;
        self.with_nonce(nonce)
    }

    /// Sets a new random hash
    pub fn rng_hash(mut self) -> Self {
        self.with_hash(H256::random())
    }

    /// Returns a new transaction with a higher gas price +1
    pub fn inc_price(&self) -> Self {
        self.inc_price_by(1)
    }

    /// Returns a new transaction with a higher gas price
    pub fn inc_price_by(&self, value: u128) -> Self {
        let mut next = self.clone();
        let gas = self.get_gas_price().checked_add(value).unwrap();
        next.with_gas_price(gas)
    }

    /// Returns a new transaction with a lower gas price -1
    pub fn decr_price(&self) -> Self {
        self.decr_price_by(1)
    }

    /// Returns a new transaction with a lower gas price
    pub fn decr_price_by(&self, value: u128) -> Self {
        let mut next = self.clone();
        let gas = self.get_gas_price().checked_sub(value).unwrap();
        next.with_gas_price(gas)
    }

    /// Returns a new transaction with a higher value
    pub fn inc_value(&self) -> Self {
        let mut next = self.clone();
        let val = self.get_value().checked_add(U256::from(1)).unwrap();
        next.with_value(val)
    }

    /// Returns a new transaction with a higher gas limit
    pub fn inc_limit(&self) -> Self {
        let mut next = self.clone();
        let gas = self.get_gas_limit() + 1;
        next.with_gas_limit(gas)
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, MockTransaction::Legacy { .. })
    }

    pub fn is_eip1559(&self) -> bool {
        matches!(self, MockTransaction::Eip1559 { .. })
    }
}

impl PoolTransaction for MockTransaction {
    fn hash(&self) -> &TxHash {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { hash, .. } => hash,
            MockTransaction::Legacy { hash, .. } => hash,
            MockTransaction::Eip1559 { hash, .. } => hash,
        }
    }

    fn sender(&self) -> Address {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { sender, .. } => *sender,
            MockTransaction::Legacy { sender, .. } => *sender,
            MockTransaction::Eip1559 { sender, .. } => *sender,
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { nonce, .. } => *nonce,
            MockTransaction::Legacy { nonce, .. } => *nonce,
            MockTransaction::Eip1559 { nonce, .. } => *nonce,
        }
    }

    fn cost(&self) -> U256 {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => U256::from(0),
            MockTransaction::Legacy { gas_price, value, gas_limit, .. } => {
                U256::from(*gas_limit) * U256::from(*gas_price) + *value
            }
            MockTransaction::Eip1559 { max_fee_per_gas, value, gas_limit, .. } => {
                U256::from(*gas_limit) * U256::from(*max_fee_per_gas) + *value
            }
        }
    }

    fn gas_cost(&self) -> U256 {
        match self {
            MockTransaction::Legacy { gas_price, gas_limit, .. } => {
                U256::from(*gas_limit) * U256::from(*gas_price)
            }
            MockTransaction::Eip1559 { max_fee_per_gas, gas_limit, .. } => {
                U256::from(*gas_limit) * U256::from(*max_fee_per_gas)
            }
        }
    }

    fn gas_limit(&self) -> u64 {
        self.get_gas_limit()
    }

    fn max_fee_per_gas(&self) -> u128 {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => None,
            MockTransaction::Legacy { .. } => None,
            MockTransaction::Eip1559 { max_fee_per_gas, .. } => Some(*max_fee_per_gas),
        }
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => None,
            MockTransaction::Legacy { .. } => None,
            MockTransaction::Eip1559 { max_priority_fee_per_gas, .. } => {
                Some(*max_priority_fee_per_gas)
            }
        }
    }

    fn kind(&self) -> &TransactionKind {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { to, .. } => to,
            MockTransaction::Legacy { to, .. } => to,
            MockTransaction::Eip1559 { to, .. } => to,
        }
    }

    fn size(&self) -> usize {
        0
    }

    fn tx_type(&self) -> u8 {
        match self {
            #[cfg(feature = "optimism")]
            MockTransaction::DepositTx { .. } => DEPOSIT_TX_TYPE,
            MockTransaction::Legacy { .. } => TxType::Legacy.into(),
            MockTransaction::Eip1559 { .. } => TxType::EIP1559.into(),
        }
    }

    fn encoded_length(&self) -> usize {
        0
    }

    fn chain_id(&self) -> Option<u64> {
        Some(1)
    }
}

impl FromRecoveredTransaction for MockTransaction {
    fn from_recovered_transaction(tx: TransactionSignedEcRecovered) -> Self {
        let sender = tx.signer();
        let transaction = tx.into_signed();
        let hash = transaction.hash();
        match transaction.transaction {
            #[cfg(feature = "optimism")]
            Transaction::Deposit(TxDeposit {
                source_hash,
                from,
                to,
                mint,
                value,
                gas_limit,
                is_system_transaction,
                input,
            }) => MockTransaction::DepositTx {
                nonce: 0u64,
                hash: source_hash,
                sender: from,
                to,
                mint,
                value: U256::from(value),
                gas_limit,
                is_system_transaction,
                input,
            },
            Transaction::Legacy(TxLegacy {
                chain_id,
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                input,
            }) => MockTransaction::Legacy {
                hash,
                sender,
                nonce,
                gas_price,
                gas_limit,
                to,
                value: U256::from(value),
            },
            Transaction::Eip1559(TxEip1559 {
                chain_id,
                nonce,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                to,
                value,
                input,
                access_list,
            }) => MockTransaction::Eip1559 {
                hash,
                sender,
                nonce,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                gas_limit,
                to,
                value: U256::from(value),
            },
            Transaction::Eip2930 { .. } => {
                unimplemented!()
            }
        }
    }
}

impl IntoRecoveredTransaction for MockTransaction {
    fn to_recovered_transaction(&self) -> TransactionSignedEcRecovered {
        let tx = Transaction::Legacy(TxLegacy {
            chain_id: self.chain_id(),
            nonce: self.get_nonce(),
            gas_price: self.get_gas_price(),
            gas_limit: self.get_gas_limit(),
            to: TransactionKind::Call(Address::from_slice(
                &hex::decode("d3e8763675e4c425df46cc3b5c0f6cbdac396046").unwrap()[..],
            )),
            value: 693361000000000u64.into(),
            input: Default::default(),
        });

        let signed_tx = TransactionSigned {
            hash: *self.hash(),
            signature: Signature::default(),
            transaction: tx,
        };

        TransactionSignedEcRecovered::from_signed_transaction(signed_tx, self.sender())
    }
}

#[derive(Default)]
pub struct MockTransactionFactory {
    pub(crate) ids: SenderIdentifiers,
}

// === impl MockTransactionFactory ===

impl MockTransactionFactory {
    pub fn tx_id(&mut self, tx: &MockTransaction) -> TransactionId {
        let sender = self.ids.sender_id_or_create(tx.get_sender());
        TransactionId::new(sender, tx.get_nonce())
    }

    pub fn validated(&mut self, transaction: MockTransaction) -> MockValidTx {
        self.validated_with_origin(TransactionOrigin::External, transaction)
    }
    pub fn validated_arc(&mut self, transaction: MockTransaction) -> Arc<MockValidTx> {
        Arc::new(self.validated(transaction))
    }

    /// Converts the transaction into a validated transaction
    pub fn validated_with_origin(
        &mut self,
        origin: TransactionOrigin,
        transaction: MockTransaction,
    ) -> MockValidTx {
        let transaction_id = self.tx_id(&transaction);
        let encoded_length = transaction.encoded_length();
        MockValidTx {
            propagate: false,
            transaction_id,
            transaction,
            timestamp: Instant::now(),
            origin,
            encoded_length,
        }
    }

    pub fn create_legacy(&mut self) -> MockValidTx {
        self.validated(MockTransaction::legacy())
    }

    pub fn create_eip1559(&mut self) -> MockValidTx {
        self.validated(MockTransaction::eip1559())
    }
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct MockOrdering;

impl TransactionOrdering for MockOrdering {
    type Priority = U256;
    type Transaction = MockTransaction;

    fn priority(&self, transaction: &Self::Transaction) -> Self::Priority {
        transaction.gas_cost()
    }
}

/// A configured distribution that can generate transactions
pub struct MockTransactionDistribution {
    /// legacy to EIP-1559 ration
    legacy_ratio: WeightedIndex<u32>,
    /// generates the gas limit
    gas_limit_range: Uniform<u64>,
}

impl MockTransactionDistribution {
    /// Creates a new generator distribution.
    ///
    /// Expects legacy tx in full pct: `30u32` is `30%`.
    pub fn new(legacy_pct: u32, gas_limit_range: Range<u64>) -> Self {
        assert!(legacy_pct <= 100, "expect pct");

        let eip_1559 = 100 - legacy_pct;
        Self {
            legacy_ratio: WeightedIndex::new([eip_1559, legacy_pct]).unwrap(),
            gas_limit_range: gas_limit_range.into(),
        }
    }

    /// Generates a new transaction
    pub fn tx(&self, nonce: u64, rng: &mut impl rand::Rng) -> MockTransaction {
        let tx = if self.legacy_ratio.sample(rng) == 0 {
            MockTransaction::eip1559()
        } else {
            MockTransaction::legacy()
        };
        tx.with_nonce(nonce).with_gas_limit(self.gas_limit_range.sample(rng))
    }
}

#[test]
fn test_mock_priority() {
    let o = MockOrdering;
    let lo = MockTransaction::eip1559().with_gas_limit(100_000);
    let hi = lo.next().inc_price();
    assert!(o.priority(&hi) > o.priority(&lo));
}
