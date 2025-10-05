use op_revm::transaction::abstraction::OpBuildError;
use op_revm::L1BlockInfo;
use op_revm::OpContext;
use op_revm::OpEvm;
use op_revm::OpHaltReason;
use op_revm::OpSpecId;
use op_revm::OpTransaction;
use revm::context::result::ExecutionResult;
use revm::context::result::HaltReason;
use revm::context::tx::TxEnvBuildError;
use revm::context::BlockEnv;
use revm::context::CfgEnv;
use revm::context::JournalTr;
use revm::context::LocalContext;
use revm::handler::instructions::EthInstructions;
use revm::handler::EthPrecompiles;
use revm::primitives::HashMap;
use revm::primitives::TxKind;
use revm::primitives::B256;
use revm::primitives::U256;
use revm::ExecuteEvm;
use revm::Journal;
use revm::MainnetEvm;
use revm::InspectEvm;

use serde_json::Value;
use std::str::FromStr;

use revm::{
    context::TxEnv, database::InMemoryDB,
    primitives::{Address, Bytes},
    Context, MainContext
};

use crate::database::create_in_memory_database_from_prestate_trace;
use crate::database::AccountDetails;
use crate::inspector::MyInspector;

pub fn trace_transaction(
    chain_id: u64,
    from: Address,
    from_nonce: u64,
    to: Address,
    data: Bytes,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: BlockEnv,
    prestate_tracer_result: HashMap<Address, AccountDetails>
) -> Result<
        (
            ExecutionResult<HaltReason>,
            HashMap<Address, revm::state::Account>, //state diff
            Value  //tracer result
        ),
        String // error
> {
    let tx_env_build = TxEnv::builder()
        .chain_id(Some(chain_id))
        .caller(from)
        .kind(TxKind::Call(to))
        .nonce(from_nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .gas_priority_fee(Some(gas_priority_fee))
        .data(data)
        .build();

    let tx: TxEnv;
    match tx_env_build{
        Ok(result) => {tx = result}
        Err(error) => {
            match error {
                TxEnvBuildError::DeriveErr(_) => {
                    return Err(String::from_str(
                            "TxEnvBuildError: Derive Error"
                        ).unwrap()
                    )
                }
                TxEnvBuildError::MissingGasPriorityFeeForEip1559 => {
                    return Err(
                        String::from_str(
                            "TxEnvBuildError: MissingGasPriorityFeeForEip1559"
                        ).unwrap()
                    )
                }
                TxEnvBuildError::MissingTargetForEip4844 => {
                    return Err(
                        String::from_str(
                            "TxEnvBuildError: MissingTargetForEip4844"
                        ).unwrap()
                    )
                }
                TxEnvBuildError::MissingAuthorizationListForEip7702 => {
                    return Err(
                        String::from_str(
                            "TxEnvBuildError: MissingAuthorizationListForEip7702"
                        ).unwrap()
                    )
                }
                TxEnvBuildError::MissingBlobHashesForEip4844 => {
                    return Err(
                        String::from_str(
                            "TxEnvBuildError: MissingBlobHashesForEip4844"
                        ).unwrap()
                    )
                }
            };
        }
    };
    
    let buffer = &mut Vec::new();
    let inspector = MyInspector::new(buffer);

    let db:InMemoryDB = create_in_memory_database_from_prestate_trace(
        prestate_tracer_result
    );

    let cfg_env = CfgEnv::new().with_chain_id(chain_id);
    let context = Context::mainnet().with_db(db).
        with_cfg(cfg_env).
        with_block(latest_block_env);

    let mut my_evm = MainnetEvm::new_with_inspector(
        context,
        inspector,
        EthInstructions::new_mainnet(),
        EthPrecompiles::default()
    );
    let execution_result = match my_evm.inspect_one_tx(tx){
        Ok(result) => {result},
        Err(error) => {return Err(error.to_string())}
    };
    let state_diff = my_evm.finalize();
    let trace_result;
    match buffer.pop() {
        Some(result) => {
            assert!(
                buffer.len() == 0,
                "invalid stack buffer result. should only have one element."
            );
            assert!(
                result.0 == 1,
                "invalid stack buffer result. stack depth should be 1."
            );

            trace_result = result.1
        }
        None => {
            //can happend with failed execution
            trace_result = Value::Null;
        }
    }

    Ok((execution_result, state_diff, trace_result))
}

pub fn op_trace_transaction(
    chain_id: u64,
    from: Address,
    from_nonce: u64,
    to: Address,
    data: Bytes,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: BlockEnv,
    prestate_tracer_result: HashMap<Address, AccountDetails>
) -> Result<
        (
            ExecutionResult<OpHaltReason>,
            HashMap<Address, revm::state::Account>, //state diff
            Value  //tracer result
        ),
        String // error
> {
    let base_tx = TxEnv::builder()
        .chain_id(Some(chain_id))
        .caller(from)
        .kind(TxKind::Call(to))
        .nonce(from_nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .gas_priority_fee(Some(gas_priority_fee))
        .data(data);
    let op_tx_build = OpTransaction::builder()
        .base(base_tx)
        .enveloped_tx(None)
        .not_system_transaction()
        .mint(0u128)
        .source_hash(B256::from([1u8; 32]))
        .build();

    let op_tx; 
    match op_tx_build{
        Ok(result) => {op_tx = result}
        Err(error) => {
            match error {
                OpBuildError::Base(_) => {
                    return Err(String::from_str("OPTxEnvBuildError: Base").unwrap())
                }
                OpBuildError::MissingEnvelopedTxBytes => {
                    return Err(
                        String::from_str(
                            "OPTxEnvBuildError: MissingEnvelopedTxBytes"
                        ).unwrap()
                    )
                }
                OpBuildError::MissingSourceHashForDeposit => {
                    return Err(
                        String::from_str(
                            "OPTxEnvBuildError: MissingSourceHashForDeposit"
                        ).unwrap()
                    )
                }
            };
        }
    };

    let cfg_env = CfgEnv::new().with_chain_id(chain_id);
    let op_spec = OpSpecId::default();
    let mut chain = L1BlockInfo::default();
    if op_spec == OpSpecId::ISTHMUS {
        chain.operator_fee_constant = Some(U256::from(0));
        chain.operator_fee_scalar = Some(U256::from(0));
    }
    let op_cfg = cfg_env.clone().with_spec(op_spec);

    let buffer = &mut Vec::new();
    let inspector = MyInspector::new(buffer);
    let db:InMemoryDB = create_in_memory_database_from_prestate_trace(
        prestate_tracer_result
    );

    let op_context = OpContext {
        journaled_state: {
            let mut journal = Journal::new(db);
            // Converting SpecId into OpSpecId
            journal.set_spec_id(cfg_env.spec);
            journal
        },
        block: latest_block_env,
        cfg: op_cfg,
        tx: OpTransaction::default(),
        chain,
        local: LocalContext::default(),
        error: Ok(()),
    };

    let mut my_evm = OpEvm::new(op_context, inspector);
    let execution_result = match my_evm.inspect_one_tx(op_tx){
        Ok(result) => {result},
        Err(error) => {return Err(error.to_string())}
    };
    let state_diff = my_evm.finalize();
    let trace_result;
    match buffer.pop() {
        Some(result) => {
            assert!(
                buffer.len() == 0,
                "invalid stack buffer result. should only have one element."
            );
            assert!(
                result.0 == 1,
                "invalid stack buffer result. stack depth should be 1."
            );

            trace_result = result.1
        }
        None => {
            //can happend with failed execution
            trace_result = Value::Null;
        }
    }

    Ok((execution_result, state_diff, trace_result))
}
