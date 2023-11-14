#![no_main]

mod utils {
    pub mod arbitrary_types;
    pub mod writer;
}

use casper_finality_proofs::test_engine::utils::data_generation::{init_beacon_state, Balances};
use casper_finality_proofs::test_engine::wrappers::wrapper_weigh_justification_and_finalization::{
    run, CIRCUIT,
};
use casper_finality_proofs::to_string;
use casper_finality_proofs::types::{BeaconTreeHashCacheType, ChainSpecType, Eth1Type};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;
use serde_derive::Serialize;
use utils::arbitrary_types::ArbitraryH256;

use crate::utils::writer::json_write;

#[derive(Debug, Clone, arbitrary::Arbitrary, Serialize)]
struct TestData {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX))]
    pub slot: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=2))]
    pub current_epoch_sub: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(2..=3))]
    pub previous_epoch_sub: u64,
    pub justification_bits: [bool; 4],
    pub current_epoch_root: ArbitraryH256,
    pub previous_epoch_root: ArbitraryH256,
    pub current_idx_root: ArbitraryH256,
    pub previous_idx_root: ArbitraryH256,
    pub finalized_checkpoint_root: ArbitraryH256,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX))]
    pub finalized_checkpoint_epoch: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX))]
    pub total_active_balance: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX))]
    pub previous_target_balance: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX))]
    pub current_target_balance: u64,
    pub eth_1_data: Eth1Type,
    #[serde(skip)]
    pub chain_spec: ChainSpecType,
}

fuzz_target!(|data: TestData| {
    Lazy::force(&CIRCUIT);

    let mut data = data;
    let epoch = data.slot / 32;
    data.chain_spec.genesis_slot = data.slot.into();
    let mut state = init_beacon_state(data.eth_1_data.clone(), &data.chain_spec);

    *state.tree_hash_cache_mut() = BeaconTreeHashCacheType::new(&state);

    let prev_checkpoint_index = (epoch - 1) * 32 % 8192;
    let curr_checkpoint_index = epoch * 32 % 8192;

    state.block_roots_mut()[prev_checkpoint_index as usize] = data.previous_idx_root.0;
    state.block_roots_mut()[curr_checkpoint_index as usize] = data.current_idx_root.0;

    state.previous_justified_checkpoint_mut().epoch = (epoch - data.previous_epoch_sub).into();
    state.previous_justified_checkpoint_mut().root = data.previous_epoch_root.0;

    state.current_justified_checkpoint_mut().epoch = (epoch - data.current_epoch_sub).into();
    state.current_justified_checkpoint_mut().root = data.current_epoch_root.0;

    state.finalized_checkpoint_mut().epoch = data.finalized_checkpoint_epoch.into();
    state.finalized_checkpoint_mut().root = data.finalized_checkpoint_root.0;

    for (i, bit) in data.justification_bits.iter().enumerate() {
        state.justification_bits_mut().set(i, *bit).unwrap();
    }

    let balances = Balances {
        total_active_balance: data.total_active_balance,
        previous_target_balance: data.previous_target_balance,
        current_target_balance: data.current_target_balance,
    };

    let output = run(&mut state, balances);

    let mut value = serde_json::json!({ "input": { "state": state, "additional_data": output.1 }, "output": {} });

    value["output"]["new_previous_justified_checkpoint"]["epoch"] =
        serde_json::Value::String(output.0.new_previous_justified_checkpoint.epoch.to_string());
    value["output"]["new_previous_justified_checkpoint"]["root"] =
        serde_json::Value::String(to_string!(output.0.new_previous_justified_checkpoint.root));
    value["output"]["new_current_justified_checkpoint"]["epoch"] =
        serde_json::Value::String(output.0.new_current_justified_checkpoint.epoch.to_string());
    value["output"]["new_current_justified_checkpoint"]["root"] =
        serde_json::Value::String(to_string!(output.0.new_current_justified_checkpoint.root));
    value["output"]["new_finalized_checkpoint"]["epoch"] =
        serde_json::Value::String(output.0.new_finalized_checkpoint.epoch.to_string());
    value["output"]["new_finalized_checkpoint"]["root"] =
        serde_json::Value::String(to_string!(output.0.new_finalized_checkpoint.root));
    value["output"]["new_justification_bits"]["bits"] = serde_json::Value::Array(
        output
            .0
            .new_justification_bits
            .bits
            .iter()
            .map(|x| serde_json::Value::Bool(*x))
            .collect(),
    );

    unsafe {
        let _ = json_write("weigh_justification_and_finalization".to_owned(), value);
    }
});
