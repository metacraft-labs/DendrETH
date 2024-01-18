use crate::{
    utils::{plonky2x_extensions::assert_is_true, eth_objects::Attestation},
    weigh_justification_and_finalization::{
        checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable
    }, constants::{VALIDATOR_ROOT_GINDEX, STATE_ROOT_GINDEX}, combine_finality_votes::circuit::ProofWithPublicInputsTargetReader,
};
use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild},
    prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable}, frontend::vars::Bytes32Variable,
};

#[derive(Debug, Clone)]
pub struct ProveFinality;

impl ProveFinality {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        vad_circuit: &CircuitBuild<L, D>,
        cuv_circuit: &CircuitBuild<L, D>
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {

        println!("HI FROM INSIDE");

        let verifier_vad = builder.constant_verifier_data::<L>(&vad_circuit.data);
        let verifier_cuv = builder.constant_verifier_data::<L>(&cuv_circuit.data);

        println!("HI FROM INSIDE");

        let vad_proof = builder.proof_read(&vad_circuit.data.common).into();
        let cuv_proof = builder.proof_read(&cuv_circuit.data.common).into();

        println!("HI FROM INSIDE");

        builder.verify_proof::<L>(&vad_proof, &verifier_vad, &vad_circuit.data.common);
        builder.verify_proof::<L>(&cuv_proof, &verifier_cuv, &cuv_circuit.data.common);

        println!("HI FROM INSIDE");

        let mut vad_proof_reader = ProofWithPublicInputsTargetReader::from(vad_proof);
        let mut cuv_proof_reader = ProofWithPublicInputsTargetReader::from(cuv_proof);

        // let prev_block_root = builder.read::<Bytes32Variable>();
        // let attestation = Attestation::circuit_input(builder); //TODO: Only some of these should come from VAD recurssive proof

        // let source = builder.read::<CheckpointVariable>(); //TODO: This should come from VAD recurssive proof
        // let target = builder.read::<CheckpointVariable>(); //TODO: This should come from VAD recurssive proof
        let slot = builder.read::<U64Variable>();
        let total_number_of_validators = builder.read::<U64Variable>();
        let justification_bits = builder.read::<JustificationBitsVariable>(); // Comes from beacon_state
        let previous_epoch_attested_validators = builder.read::<U64Variable>();
        let current_epoch_attested_validators = builder.read::<U64Variable>();
        let previous_justified_checkpoint = builder.read::<CheckpointVariable>(); // Comes from beacon_state
        let current_justified_checkpoint = builder.read::<CheckpointVariable>(); // Comes from beacon_state

        // block_merkle_branch_proof(
        //     builder,
        //     prev_block_root,
        //     attestation.clone()
        // );

        // validate_target_source_difference(builder, source.clone(), target.clone());

        // let new_justification_bits = process_justifications(
        //     builder,
        //     total_number_of_validators,
        //     justification_bits,
        //     previous_epoch_attested_validators,
        //     current_epoch_attested_validators,
        // );

        // let thirty_two = builder.constant::<U64Variable>(32);
        // let new_justification_bits = new_justification_bits.bits.as_slice();
        // let current_epoch = builder.div(slot, thirty_two);
        // let source_index = builder.sub(current_epoch, source.epoch);
        // let target_index = builder.sub(current_epoch, target.epoch);

        // validate_source(
        //     builder,
        //     source,
        //     target_index,
        //     previous_justified_checkpoint,
        //     current_justified_checkpoint,
        // );

        // validate_justification_bits(builder, source_index, target_index, new_justification_bits);
    }
}

fn block_merkle_branch_proof<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    prev_block_root: Bytes32Variable,
    attestation: Attestation
) {
    // let field_eleven = <L as PlonkParameters<D>>::Field::from_canonical_u64(11);
    let const11 = builder.constant(VALIDATOR_ROOT_GINDEX as u64);
    let const43 = builder.constant(STATE_ROOT_GINDEX as u64);

    // Verify that the given `state_root` is in the last trusted `block_root`
    builder.ssz_verify_proof(
        prev_block_root,
        attestation.state_root,
        attestation.state_root_proof.as_slice(),
        const11
    );

    /*
    Verify that the `validators_root` is within the already verified
    `state_root`.  All validators will be verified against this
    `validators_root`.
    */
    builder.ssz_verify_proof(
        attestation.state_root,
        attestation.validators_root,
        attestation.validators_root_proof.as_slice(),
        const43
    )

}

fn process_justifications<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_number_of_validators: U64Variable,
    justification_bits: JustificationBitsVariable,
    previous_epoch_attested_validators: U64Variable,
    current_epoch_attested_validators: U64Variable,
) -> JustificationBitsVariable {
    let previous_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        total_number_of_validators,
        previous_epoch_attested_validators,
    );

    let current_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        total_number_of_validators,
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
    source: CheckpointVariable,
    target: CheckpointVariable,
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
