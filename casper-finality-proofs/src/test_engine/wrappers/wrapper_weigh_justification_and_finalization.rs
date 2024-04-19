use crate::{
    assert_equal,
    constants::{
        BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX, BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX,
        BEACON_STATE_JUSTIFICATION_BITS_GINDEX, BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX,
        BEACON_STATE_SLOT_GINDEX,
    },
    test_engine::utils::{
        data_generation::{
            compute_beacon_state_tree_hash_root,
            compute_block_roots_start_epoch_slot_to_beacon_state_proof, compute_merkle_proof,
            extract_balances, get_block_root_epoch_start_slot_root, Balances,
        },
        parsers::ssz_decoder::read_ssz_fixture,
    },
    weigh_justification_and_finalization::{
        checkpoint::{CheckpointValue, CheckpointVariable},
        justification_bits::{JustificationBitsValue, JustificationBitsVariable},
        WeighJustificationAndFinalization,
    },
};
use lighthouse_ef_tests::{self, testing_spec};
use lighthouse_types::{BeaconState, ForkName, MainnetEthSpec};
use once_cell::sync::Lazy;
use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild},
    prelude::{
        ArrayVariable, Bytes32Variable, CircuitBuilder, DefaultParameters, GoldilocksField,
        PlonkParameters, U64Variable,
    },
};
use primitive_types::H256;
use serde_derive::Serialize;

// Singleton-like pattern
pub static CIRCUIT: Lazy<CircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    builder.build()
});

#[derive(Debug)]
pub struct CircuitValues {
    pub new_previous_justified_checkpoint: CheckpointValue<GoldilocksField>,
    pub new_current_justified_checkpoint: CheckpointValue<GoldilocksField>,
    pub new_finalized_checkpoint: CheckpointValue<GoldilocksField>,
    pub new_justification_bits: JustificationBitsValue<GoldilocksField>,
}

#[derive(Debug, Serialize)]
pub struct StateData {
    pub slot_proof: Vec<H256>,
    pub beacon_state_root: H256,
    pub previous_justified_checkpoint_proof: Vec<H256>,
    pub current_justified_checkpoint_proof: Vec<H256>,
    pub justification_bits_proof: Vec<H256>,
    pub previous_epoch_start_slot_root_in_block_roots_proof: Vec<H256>,
    pub current_epoch_start_slot_root_in_block_roots_proof: Vec<H256>,
    pub previous_epoch_start_slot_root_in_block_roots: H256,
    pub current_epoch_start_slot_root_in_block_roots: H256,
    pub finalized_checkpoint_proof: Vec<H256>,
}

pub fn wrapper(path: String, should_assert: bool) -> Result<String, anyhow::Error> {
    let spec = &testing_spec::<MainnetEthSpec>(ForkName::Capella);
    let mut state = read_ssz_fixture::<MainnetEthSpec>(
        String::from(path.clone() + "/pre.ssz_snappy").as_str(),
        spec,
    );
    state.initialize_tree_hash_cache();
    let balances = extract_balances(&mut state, spec);

    let post_state = read_ssz_fixture::<MainnetEthSpec>(
        String::from(path.clone() + "/post.ssz_snappy").as_str(),
        spec,
    );

    let output = run(&mut state, balances);

    if should_assert {
        assert_equal!(
            output.0.new_previous_justified_checkpoint.epoch,
            post_state.previous_justified_checkpoint().epoch.as_u64()
        );
        assert_equal!(
            output.0.new_current_justified_checkpoint.epoch,
            post_state.current_justified_checkpoint().epoch.as_u64()
        );
        assert_equal!(
            output.0.new_current_justified_checkpoint.root,
            post_state.current_justified_checkpoint().root
        );
        assert_equal!(
            output.0.new_finalized_checkpoint.epoch,
            post_state.finalized_checkpoint().epoch.as_u64()
        );
        assert_equal!(
            output.0.new_finalized_checkpoint.root,
            post_state.finalized_checkpoint().root
        );
        assert_equal!(
            output.0.new_justification_bits.bits,
            post_state
                .justification_bits()
                .iter()
                .map(|byte| byte as bool)
                .collect::<Vec<bool>>()
        );
    }

    Ok(format!("{:?}", output))
}

pub fn run(
    state: &mut BeaconState<MainnetEthSpec>,
    balances: Balances,
) -> (CircuitValues, StateData) {
    type L = DefaultParameters;
    const D: usize = 2;

    let slot = state.slot().as_u64();
    let slot_proof = compute_merkle_proof(state, BEACON_STATE_SLOT_GINDEX as usize);

    let beacon_state_root = compute_beacon_state_tree_hash_root(state);

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.previous_justified_checkpoint().epoch.as_u64(),
        root: state.previous_justified_checkpoint().root,
    };

    let previous_justified_checkpoint_proof = compute_merkle_proof(
        state,
        BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX as usize,
    );

    let current_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.current_justified_checkpoint().epoch.as_u64(),
        root: state.current_justified_checkpoint().root,
    };

    let current_justified_checkpoint_proof = compute_merkle_proof(
        state,
        BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX as usize,
    );

    let justification_bits = JustificationBitsValue::<<L as PlonkParameters<D>>::Field> {
        bits: state
            .justification_bits()
            .iter()
            .map(|byte| byte as bool)
            .collect(),
    };

    let justification_bits_proof =
        compute_merkle_proof(state, BEACON_STATE_JUSTIFICATION_BITS_GINDEX as usize);

    let previous_epoch = state.previous_epoch();
    let previous_epoch_start_slot_root_in_block_roots_proof =
        compute_block_roots_start_epoch_slot_to_beacon_state_proof(state, previous_epoch);

    let current_epoch = state.current_epoch();
    let current_epoch_start_slot_root_in_block_roots_proof =
        compute_block_roots_start_epoch_slot_to_beacon_state_proof(state, current_epoch);

    let previous_epoch_start_slot_root_in_block_roots =
        get_block_root_epoch_start_slot_root(&state, state.previous_epoch());
    let current_epoch_start_slot_root_in_block_roots =
        get_block_root_epoch_start_slot_root(&state, state.current_epoch());

    let finalized_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.finalized_checkpoint().epoch.as_u64(),
        root: state.finalized_checkpoint().root,
    };

    let finalized_checkpoint_proof =
        compute_merkle_proof(state, BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX as usize);

    let mut input = CIRCUIT.input();

    input.write::<Bytes32Variable>(beacon_state_root);
    input.write::<U64Variable>(slot);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(slot_proof.to_vec());
    input.write::<CheckpointVariable>(previous_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(previous_justified_checkpoint_proof.to_vec());
    input.write::<CheckpointVariable>(current_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(current_justified_checkpoint_proof.to_vec());
    input.write::<JustificationBitsVariable>(justification_bits);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(justification_bits_proof.to_vec());
    input.write::<U64Variable>(balances.total_active_balance);
    input.write::<U64Variable>(balances.previous_target_balance);
    input.write::<U64Variable>(balances.current_target_balance);
    input.write::<Bytes32Variable>(previous_epoch_start_slot_root_in_block_roots);
    input.write::<ArrayVariable<Bytes32Variable, 18>>(
        previous_epoch_start_slot_root_in_block_roots_proof.to_vec(),
    );
    input.write::<Bytes32Variable>(current_epoch_start_slot_root_in_block_roots);
    input.write::<ArrayVariable<Bytes32Variable, 18>>(
        current_epoch_start_slot_root_in_block_roots_proof.to_vec(),
    );
    input.write::<CheckpointVariable>(finalized_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(finalized_checkpoint_proof.to_vec());

    let (proof, mut output) = CIRCUIT.prove(&input);
    CIRCUIT.verify(&proof, &input, &output);

    let new_previous_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_current_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_finalized_checkpoint = output.read::<CheckpointVariable>();
    let new_justification_bits = output.read::<JustificationBitsVariable>();

    (
        CircuitValues {
            new_previous_justified_checkpoint,
            new_current_justified_checkpoint,
            new_finalized_checkpoint,
            new_justification_bits,
        },
        StateData {
            slot_proof,
            beacon_state_root,
            previous_justified_checkpoint_proof,
            current_justified_checkpoint_proof,
            justification_bits_proof,
            previous_epoch_start_slot_root_in_block_roots_proof,
            current_epoch_start_slot_root_in_block_roots_proof,
            previous_epoch_start_slot_root_in_block_roots,
            current_epoch_start_slot_root_in_block_roots,
            finalized_checkpoint_proof,
        },
    )
}
