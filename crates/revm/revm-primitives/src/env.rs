use crate::config::revm_spec;
use reth_primitives::{
    Address, ChainSpec, Head, Header, Transaction, TransactionKind, TransactionSignedEcRecovered,
    TxEip1559, TxEip2930, TxLegacy, U256,
};
use revm::primitives::{AnalysisKind, BlockEnv, CfgEnv, SpecId, TransactTo, TxEnv};

#[cfg(feature = "optimism")]
use reth_primitives::TxDeposit;

/// Convenience function to call both [fill_cfg_env] and [fill_block_env]
pub fn fill_cfg_and_block_env(
    cfg: &mut CfgEnv,
    block_env: &mut BlockEnv,
    chain_spec: &ChainSpec,
    header: &Header,
    total_difficulty: U256,
) {
    fill_cfg_env(cfg, chain_spec, header, total_difficulty);
    let after_merge = cfg.spec_id >= SpecId::MERGE;
    fill_block_env(block_env, header, after_merge);
}

/// Fill [CfgEnv] fields according to the chain spec and given header
pub fn fill_cfg_env(
    cfg_env: &mut CfgEnv,
    chain_spec: &ChainSpec,
    header: &Header,
    total_difficulty: U256,
) {
    let spec_id = revm_spec(
        chain_spec,
        Head {
            number: header.number,
            timestamp: header.timestamp,
            difficulty: header.difficulty,
            total_difficulty,
            hash: Default::default(),
        },
    );

    cfg_env.chain_id = U256::from(chain_spec.chain().id());
    cfg_env.spec_id = spec_id;
    cfg_env.perf_all_precompiles_have_balance = false;
    cfg_env.perf_analyse_created_bytecodes = AnalysisKind::Analyse;
}
/// Fill block environment from Block.
pub fn fill_block_env(block_env: &mut BlockEnv, header: &Header, after_merge: bool) {
    block_env.number = U256::from(header.number);
    block_env.coinbase = header.beneficiary;
    block_env.timestamp = U256::from(header.timestamp);
    if after_merge {
        block_env.prevrandao = Some(header.mix_hash);
        block_env.difficulty = U256::ZERO;
    } else {
        block_env.difficulty = header.difficulty;
        block_env.prevrandao = None;
    }
    block_env.basefee = U256::from(header.base_fee_per_gas.unwrap_or_default());
    block_env.gas_limit = U256::from(header.gas_limit);
}

/// Returns a new [TxEnv] filled with the transaction's data.
pub fn tx_env_with_recovered(transaction: &TransactionSignedEcRecovered) -> TxEnv {
    let mut tx_env = TxEnv::default();
    fill_tx_env(&mut tx_env, transaction.as_ref(), transaction.signer());
    tx_env
}

/// Fill transaction environment from [TransactionSignedEcRecovered].
pub fn fill_tx_env_with_recovered(tx_env: &mut TxEnv, transaction: &TransactionSignedEcRecovered) {
    fill_tx_env(tx_env, transaction.as_ref(), transaction.signer())
}

/// Fill transaction environment from a [Transaction] and the given sender address.
pub fn fill_tx_env<T>(tx_env: &mut TxEnv, transaction: T, sender: Address)
where
    T: AsRef<Transaction>,
{
    tx_env.caller = sender;
    match transaction.as_ref() {
        #[cfg(feature = "optimism")]
        Transaction::Deposit(TxDeposit { to, mint, value, gas_limit, input, .. }) => {
            tx_env.gas_limit = *gas_limit;
            if let Some(m) = mint {
                tx_env.gas_price = U256::from(*m);
            } else {
                tx_env.gas_price = U256::ZERO;
            }
            tx_env.gas_priority_fee = None;
            match to {
                TransactionKind::Call(to) => tx_env.transact_to = TransactTo::Call(*to),
                TransactionKind::Create => tx_env.transact_to = TransactTo::create(),
            }
            tx_env.value = U256::from(*value);
            tx_env.data = input.0.clone();
            tx_env.chain_id = None;
            tx_env.nonce = None;
        }
        Transaction::Legacy(TxLegacy {
            nonce,
            chain_id,
            gas_price,
            gas_limit,
            to,
            value,
            input,
        }) => {
            tx_env.gas_limit = *gas_limit;
            tx_env.gas_price = U256::from(*gas_price);
            tx_env.gas_priority_fee = None;
            tx_env.transact_to = match to {
                TransactionKind::Call(to) => TransactTo::Call(*to),
                TransactionKind::Create => TransactTo::create(),
            };
            tx_env.value = U256::from(*value);
            tx_env.data = input.0.clone();
            tx_env.chain_id = *chain_id;
            tx_env.nonce = Some(*nonce);
            tx_env.access_list.clear();
        }
        Transaction::Eip2930(TxEip2930 {
            nonce,
            chain_id,
            gas_price,
            gas_limit,
            to,
            value,
            input,
            access_list,
        }) => {
            tx_env.gas_limit = *gas_limit;
            tx_env.gas_price = U256::from(*gas_price);
            tx_env.gas_priority_fee = None;
            tx_env.transact_to = match to {
                TransactionKind::Call(to) => TransactTo::Call(*to),
                TransactionKind::Create => TransactTo::create(),
            };
            tx_env.value = U256::from(*value);
            tx_env.data = input.0.clone();
            tx_env.chain_id = Some(*chain_id);
            tx_env.nonce = Some(*nonce);
            tx_env.access_list = access_list
                .0
                .iter()
                .map(|l| {
                    (
                        l.address,
                        l.storage_keys
                            .iter()
                            .map(|k| U256::from_be_bytes(k.to_fixed_bytes()))
                            .collect(),
                    )
                })
                .collect();
        }
        Transaction::Eip1559(TxEip1559 {
            nonce,
            chain_id,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            to,
            value,
            input,
            access_list,
        }) => {
            tx_env.gas_limit = *gas_limit;
            tx_env.gas_price = U256::from(*max_fee_per_gas);
            tx_env.gas_priority_fee = Some(U256::from(*max_priority_fee_per_gas));
            tx_env.transact_to = match to {
                TransactionKind::Call(to) => TransactTo::Call(*to),
                TransactionKind::Create => TransactTo::create(),
            };
            tx_env.value = U256::from(*value);
            tx_env.data = input.0.clone();
            tx_env.chain_id = Some(*chain_id);
            tx_env.nonce = Some(*nonce);
            tx_env.access_list = access_list
                .0
                .iter()
                .map(|l| {
                    (
                        l.address,
                        l.storage_keys
                            .iter()
                            .map(|k| U256::from_be_bytes(k.to_fixed_bytes()))
                            .collect(),
                    )
                })
                .collect();
        }
    }
}
