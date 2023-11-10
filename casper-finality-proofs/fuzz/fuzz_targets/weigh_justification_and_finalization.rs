#![no_main]

mod utils {
    pub mod arbitrary_types;
}

use casper_finality_proofs::test_engine::utils::data_generation::{init_beacon_state, Balances};
use casper_finality_proofs::test_engine::wrappers::wrapper_weigh_justification_and_finalization::{
    run, CIRCUIT,
};
use casper_finality_proofs::types::{BeaconTreeHashCacheType, ChainSpecType, Eth1Type};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;
use utils::arbitrary_types::ArbitraryH256;

#[derive(Debug, arbitrary::Arbitrary)]
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
    pub chain_spec: ChainSpecType,
}

fuzz_target!(|data: TestData| {
    Lazy::force(&CIRCUIT);

    println!("data: {:?}", data);

    let mut data = data;
    let epoch = data.slot / 32;
    data.chain_spec.genesis_slot = data.slot.into();
    let mut state = init_beacon_state(data.eth_1_data, &data.chain_spec);

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

    let (
        new_previous_justified_checkpoint,
        new_current_justified_checkpoint,
        new_finalized_checkpoint,
        new_justification_bits,
    ) = run(state, balances);

    let res = format!(
        "previous_justified_checkpoint: {:?};\n",
        new_previous_justified_checkpoint
    ) + format!(
        "current_justified_checkpoint: {:?};\n",
        new_current_justified_checkpoint
    )
    .as_str()
        + format!("finalized_checkpoint: {:?};\n", new_finalized_checkpoint).as_str()
        + format!("justification_bits: {:?};\n", new_justification_bits.bits).as_str();

    println!("res: {}", res);
});
