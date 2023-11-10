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
use lighthouse_state_processing::per_epoch_processing::{
    weigh_justification_and_finalization, JustificationAndFinalizationState,
};
use lighthouse_types::{EthSpec, MainnetEthSpec};
use once_cell::sync::Lazy;
use serde_derive::Serialize;
use utils::arbitrary_types::ArbitraryH256;

#[derive(Debug, Clone, arbitrary::Arbitrary, Serialize)]
struct TestData {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(33..=u64::MAX-2u64.pow(13)-1))]
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
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX/3))]
    pub total_active_balance: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX/3))]
    pub previous_target_balance: u64,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=u64::MAX/3))]
    pub current_target_balance: u64,
    pub eth_1_data: Eth1Type,
    #[serde(skip)]
    pub chain_spec: ChainSpecType,
}

fuzz_target!(|data: TestData| {
    let time = std::time::Instant::now();
    Lazy::force(&CIRCUIT);

    println!("data: {:?}", data);

    let mut data = data;
    let epoch = data.slot / 32;
    if epoch < data.previous_epoch_sub {
        return;
    }

    data.chain_spec.genesis_slot = data.slot.into();
    let mut state = init_beacon_state(data.eth_1_data.clone(), &data.chain_spec);

    if data.slot
        == state
            .current_epoch()
            .start_slot(MainnetEthSpec::slots_per_epoch())
            .as_u64()
    {
        // Invalid input
        return;
    }

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

    let new_state = JustificationAndFinalizationState::<MainnetEthSpec>::new(&state);
    let output_ref = weigh_justification_and_finalization(
        new_state,
        data.total_active_balance,
        data.previous_target_balance,
        data.current_target_balance,
    )
    .unwrap();

    assert!(
        output_ref.previous_justified_checkpoint().epoch
            == output.0.new_previous_justified_checkpoint.epoch
    );
    assert!(
        output_ref.previous_justified_checkpoint().root
            == output.0.new_previous_justified_checkpoint.root
    );

    assert!(
        output_ref.current_justified_checkpoint().epoch
            == output.0.new_current_justified_checkpoint.epoch
    );
    assert!(
        output_ref.current_justified_checkpoint().root
            == output.0.new_current_justified_checkpoint.root
    );

    assert!(output_ref.finalized_checkpoint().epoch == output.0.new_finalized_checkpoint.epoch);
    assert!(output_ref.finalized_checkpoint().root == output.0.new_finalized_checkpoint.root);

    output_ref
        .justification_bits()
        .iter()
        .enumerate()
        .for_each(|(i, x)| {
            assert!(x == output.0.new_justification_bits.bits[i]);
        });

    println!("test took: {:?}", time.elapsed());
});
