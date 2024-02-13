use std::marker::PhantomData;

use crate::{
    utils::{eth_objects::{BeaconState, CheckpointVariable}, plonky2x_extensions::assert_is_true},
    weigh_justification_and_finalization::{
        justification_bits::JustificationBitsVariable
    }, constants::{VALIDATOR_ROOT_GINDEX, STATE_ROOT_GINDEX, STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN}, combine_finality_votes::circuit::ProofWithPublicInputsTargetReader,
};
use ethers::core::k256::elliptic_curve::bigint::U64;
use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use plonky2x::{
    backend::circuit::CircuitBuild,
    prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable}, frontend::vars::{Bytes32Variable, ArrayVariable},
};

fn new_proof_reader<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>, circuit: &CircuitBuild<L, D>) 
    -> ProofWithPublicInputsTargetReader<D>
{
    let verifier = builder.constant_verifier_data::<L>(&circuit.data);
    let proof: ProofWithPublicInputsTarget<D> = builder.proof_read(&circuit.data.common).into();

    ProofWithPublicInputsTargetReader::from(proof)
}

pub struct VADCircuit<L: PlonkParameters<D>, const D: usize> {
    pub sigma: U64Variable,
    pub commitment: U64Variable,
    pub target: CheckpointVariable,
    pub source: CheckpointVariable,
    pub state_root: Bytes32Variable,
    pub state_root_proof: ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>,
    pub validators_root: Bytes32Variable,
    pub validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>,
    pub slot: U64Variable,
}

impl<L: PlonkParameters<D>, const D: usize> VADCircuit<L,D>{
    
    pub fn new(builder: &mut CircuitBuilder<L, D>, circuit: &CircuitBuild<L, D>) -> Self{

        let mut proof_reader = new_proof_reader(builder, circuit);

        VADCircuit {
            sigma: proof_reader.read::<U64Variable>(),
            commitment: proof_reader.read::<U64Variable>(),
            target: proof_reader.read::<CheckpointVariable>(),
            source: proof_reader.read::<CheckpointVariable>(),
            state_root: proof_reader.read::<Bytes32Variable>(),
            state_root_proof: proof_reader.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(),
            validators_root: proof_reader.read::<Bytes32Variable>(),
            validators_root_proof: proof_reader.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(),
            slot: proof_reader.read::<U64Variable>(),
        }
    }
}

pub struct CUVCircuit<L: PlonkParameters<D>, const D: usize> {
    pub rightmost: U64Variable,
    pub leftmost: U64Variable,
    pub commitment: U64Variable,
    pub total_unique_validators: U64Variable,
    _phantom: PhantomData<L>
}

impl<L: PlonkParameters<D>, const D: usize> CUVCircuit<L,D>{
    
    pub fn new(builder: &mut CircuitBuilder<L, D>, circuit: &CircuitBuild<L, D>) -> Self{

        let mut proof_reader = new_proof_reader(builder, circuit);

        CUVCircuit {
            rightmost: proof_reader.read::<U64Variable>(),
            leftmost: proof_reader.read::<U64Variable>(),
            commitment: proof_reader.read::<U64Variable>(),
            total_unique_validators: proof_reader.read::<U64Variable>(),
            _phantom: PhantomData::<L> {}
        }
    }
}
#[derive(Debug, Clone)]
pub struct ProveFinality;

impl ProveFinality {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        vad_circuit_cur: &CircuitBuild<L, D>,
        cuv_circuit_cur: &CircuitBuild<L, D>,

        vad_circuit_prev: &CircuitBuild<L, D>,
        cuv_circuit_prev: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {

        let vad_cur = VADCircuit::<L,D>::new(builder, vad_circuit_cur);
        let vad_prev = VADCircuit::<L,D>::new(builder, vad_circuit_prev);

        let cuv_cur = CUVCircuit::<L,D>::new(builder, cuv_circuit_cur);
        let cuv_prev = CUVCircuit::<L,D>::new(builder, cuv_circuit_prev);

        let beacon_state = BeaconState::circuit_input(builder);

        // let prev_block_root = builder.read::<Bytes32Variable>();
        // let active_validators_count = builder.read::<U64Variable>();

        builder.assert_is_equal(vad_cur.commitment, cuv_cur.commitment);
        builder.assert_is_equal(vad_prev.commitment, cuv_prev.commitment);

        // // Served from commitment mapper, propogated by VAD Recurssive Circuit
        // let previous_epoch_attested_validators = builder.read::<U64Variable>(); 
        // let current_epoch_attested_validators = builder.read::<U64Variable>();

        // block_merkle_branch_proof(
        //     builder,
        //     prev_block_root,
        //     vad_cur.state_root,
        //     vad_cur.validators_root,
        //     vad_cur.state_root_proof,
        //     vad_cur.validators_root_proof
        // );

        validate_target_source_difference(builder, &vad_cur.source, &vad_cur.target); //TODO: Bugged Data

        // let new_justification_bits = process_justifications(
        //     builder,
        //     active_validators_count,
        //     beacon_state.justification_bits,
        //     cuv_prev.total_unique_validators, // previous_epoch_attested_validators,
        //     cuv_cur.total_unique_validators, // current_epoch_attested_validators,
        // );

        let thirty_two = builder.constant::<U64Variable>(32);
        // let new_justification_bits = new_justification_bits.bits.as_slice();
        let current_epoch = builder.div(vad_cur.slot, thirty_two);
        let source_index = builder.sub(current_epoch, vad_cur.source.epoch);
        let target_index = builder.sub(current_epoch, vad_cur.target.epoch);

        validate_source( 
            builder,
            vad_cur.source,
            target_index,
            beacon_state.previous_justified_checkpoint,
            beacon_state.current_justified_checkpoint,
        );

        // validate_justification_bits(builder, source_index, target_index, new_justification_bits.bits.as_slice());
    }
}

fn block_merkle_branch_proof<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    prev_block_root: Bytes32Variable,
    state_root: Bytes32Variable,
    validators_root: Bytes32Variable,
    state_root_proof: ArrayVariable<Bytes32Variable,STATE_ROOT_PROOF_LEN>,
    validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>
) {
    // let field_eleven = <L as PlonkParameters<D>>::Field::from_canonical_u64(11);
    let const11 = builder.constant(VALIDATOR_ROOT_GINDEX as u64);
    let const43 = builder.constant(STATE_ROOT_GINDEX as u64);

    // Verify that the given `state_root` is in the last trusted `block_root`
    builder.ssz_verify_proof(
        prev_block_root,
        state_root,
        state_root_proof.as_slice(),
        const11
    );

    /*
    Verify that the `validators_root` is within the already verified
    `state_root`.  All validators will be verified against this
    `validators_root`.
    */
    builder.ssz_verify_proof(
        state_root,
        validators_root,
        validators_root_proof.as_slice(),
        const43
    )

}

fn process_justifications<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    active_validators_count: U64Variable,
    justification_bits: JustificationBitsVariable,
    previous_epoch_attested_validators: U64Variable,
    current_epoch_attested_validators: U64Variable,
) -> JustificationBitsVariable {
    let previous_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        active_validators_count,
        previous_epoch_attested_validators,
    );

    let current_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        active_validators_count,
        current_epoch_attested_validators,
    );

    let _true = builder._true();
    let new_second_justification_bit = builder.select(
        previous_epoch_supermajority_link_pred,
        _true,
        justification_bits.bits[0],
    );

    let mut new_justification_bits = justification_bits.shift_right(builder);
    new_justification_bits = new_justification_bits.assign_nth_bit(1, new_second_justification_bit);
    new_justification_bits =
        new_justification_bits.assign_nth_bit(0, current_epoch_supermajority_link_pred);

    new_justification_bits
}

fn is_supermajority_link_in_votes<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_num_validators: U64Variable,
    bitmask_attested_validators: U64Variable,
) -> BoolVariable {
    let five = builder.constant::<U64Variable>(5);
    let four = builder.constant::<U64Variable>(4);

    let bitmask_attested_validators_five_times = builder.mul(bitmask_attested_validators, five);
    let total_num_validators_four_times = builder.mul(total_num_validators, four);
    builder.gte(
        bitmask_attested_validators_five_times,
        total_num_validators_four_times,
    )
}

pub fn validate_target_source_difference<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source: &CheckpointVariable,
    target: &CheckpointVariable,
) {
    let one = builder.one();
    let two = builder.constant::<U64Variable>(2);

    let target_source_difference = builder.sub(target.epoch, source.epoch);
    let target_source_difference_equals_one = builder.is_equal(target_source_difference, one);
    let target_source_difference_equals_two = builder.is_equal(target_source_difference, two);
    let target_source_difference_equals_one_or_two = builder.or(
        target_source_difference_equals_one,
        target_source_difference_equals_two,
    );
    assert_is_true(builder, target_source_difference_equals_one_or_two);
}

pub fn validate_source<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source: CheckpointVariable,
    target_idx: U64Variable,
    previous_justified_checkpoint: CheckpointVariable,
    current_justified_checkpoint: CheckpointVariable,
) {
    let zero = builder.zero();
    let one = builder.one();

    let source_is_current_justified_checkpoint_pred =
        builder.is_equal(source.clone(), current_justified_checkpoint);
    let source_is_previous_justified_checkpoint_pred =
        builder.is_equal(source.clone(), previous_justified_checkpoint);

    let target_is_current_epoch_pred = builder.is_equal(target_idx, zero);
    let target_is_previous_epoch_pred = builder.is_equal(target_idx, one);

    let is_valid_pair_1_pred = builder.and(
        target_is_current_epoch_pred,
        source_is_current_justified_checkpoint_pred,
    );
    let is_valid_pair_2_pred = builder.and(
        target_is_previous_epoch_pred,
        source_is_previous_justified_checkpoint_pred,
    );
    let is_valid_pair_pred = builder.or(is_valid_pair_1_pred, is_valid_pair_2_pred);
    assert_is_true(builder, is_valid_pair_pred);
}

// Is this consistent with the consensus spec equivalent - # Process finalizations ?
pub fn validate_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source_index_epoch: U64Variable,
    target_index_epoch: U64Variable,
    justification_bits: &[BoolVariable],
) {
    for i in 0..4 {
        let current_index = builder.constant::<U64Variable>(i as u64);
        let in_range_source_index = builder.lte(current_index, source_index_epoch);
        let in_range_target_index = builder.gte(current_index, target_index_epoch);

        let in_range = builder.and(in_range_source_index, in_range_target_index);

        let in_range_or_justification_bits_value = builder.or(justification_bits[i], in_range);

        builder.assert_is_equal(justification_bits[i], in_range_or_justification_bits_value);
    }
}
