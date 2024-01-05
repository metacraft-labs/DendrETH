use curta::chip::field;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable}, frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable}, vars::{Bytes32Variable, U256Variable}, uint::uint64::U64Variable, hash::poseidon::poseidon256::PoseidonHashOutVariable},
};

use crate::utils::eth_objects::{ValidatorData, Fork, AttestationData, Attestation};
use super::super::constants::{VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_PER_COMMITTEE,VALIDATORS_ROOT_PROOF_LEN, STATE_ROOT_PROOF_LEN};
use super::super::combine_finality_votes::count_unique_pubkeys::ssz_verify_proof_poseidon;
const PLACEHOLDER: usize = 11;

#[derive(Debug, Clone)]
pub struct VerifyAttestationData;

impl Circuit for VerifyAttestationData {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {

        // //TODO: 1. Sigma <- Challenge Assrt != 0

        let prev_block_root = builder.read::<Bytes32Variable>(); 

        let attestation = Attestation::circuit_input(builder);

        //TODO: This should be part of the final proof as it is only needed once
        // 2. 3.
        block_merkle_branch_proof(
            builder,
            prev_block_root,
            attestation.clone()
        );

        //Assert that BLS Signature is correct
        // builder.assert_is_equal(attestation.signature, pk_accumulator);

        // let random_validator = builder.read::<ValidatorData>();

        // let validator_vec: Vec<ValidatorData> = (0..VALIDATORS_PER_COMMITTEE)
        //     .map(|_| ValidatorData::circuit_input(builder))
        //     .collect();

        // // Private BLS Accumulator for the recurssive proof
        // let zero_bls = validator_vec[0].beacon_validator_variable.pubkey;
        // let mut private_accumulator = validator_vec[0].beacon_validator_variable.pubkey; // TODO: validator hash
        

        // // Add validator pubkey to commitment if validator is trusted
        // for i in 1..VALIDATORS_PER_COMMITTEE {
        //         let value_to_add = builder.select(
        //             validator_vec[i].is_trusted_validator,
        //             validator_vec[i].beacon_validator_variable.pubkey, // TODO: validator hash
        //             zero_bls
        //         ); 
        //         accumulate_bls(builder,private_accumulator, value_to_add); // TODO: validator hash
        // }

        // let mut pk_accumulator = validator_vec[0].beacon_validator_variable.pubkey;
        // for i in 1..VALIDATORS_PER_COMMITTEE {

        //     // 4. Accumulate BLS Signature
        //     pk_accumulator = accumulate_bls(builder,pk_accumulator, validator_vec[i].beacon_validator_variable.pubkey);

        //     // 5. Verify Validator set
        //     verify_validator(builder, validator_vec[i].clone());
        // }

        //Will accumulate sorted validator index hash messages 
    }
}


fn block_merkle_branch_proof<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    prev_block_root: Bytes32Variable,
    attestation: Attestation
) {
    // let field_eleven = <L as PlonkParameters<D>>::Field::from_canonical_u64(11);
    let const11 = builder.constant(11 as u64);
    let const43 = builder.constant(43 as u64);

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

fn verify_validator<L: PlonkParameters<D>, const D: usize>( // TODO: Should pass only trusted_validators
    builder: &mut CircuitBuilder<L, D>,
    validator: ValidatorData

) 
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    
    // Ordering check
    let align_epoch1 = builder.lte(
        validator.beacon_validator_variable.activation_eligibility_epoch,
        validator.beacon_validator_variable.activation_epoch
    );
    let align_epoch2 = builder.lte(
        validator.beacon_validator_variable.activation_epoch, 
        validator.beacon_validator_variable.withdrawable_epoch
    );
    let align_epoch3 = builder.lte(
        validator.beacon_validator_variable.withdrawable_epoch, 
        validator.beacon_validator_variable.exit_epoch
    );

    let valid_epochs = vec![align_epoch1.variable,align_epoch2.variable, align_epoch3.variable];
    let aligned_count = builder.add_many(&valid_epochs);
    // implement add_many for BoolVariable or better yet and_many

    let field_three = <L as PlonkParameters<D>>::Field::from_canonical_u64(3);
    let const_three = builder.constant(field_three);
    builder.assert_is_equal(aligned_count, const_three);

    // Prove validator is part of the validator set

    //TODO: BeaconValidatorVariable and ValidatorData should be the same object

    builder.ssz_verify_proof(
        validator.validator_state_root,
        validator.validator_leaf,
        validator.validator_root_proof.as_slice(),
        validator.validator_gindex
    );
    // ssz_verify_proof_poseidon( //TODO: PoseidonHash
        // builder,
        // validator_hash.validator_state_root,
        // validator_hash.validator_leaf,
        // validator_hash.validator_root_proof.as_slice(),
        // validator_hash.validator_gindex
    // );
    //TODO: I need access to validator.slot to prove slot is part of beacon state [NOT RELEVANT?]

}

fn accumulate_bls<L: PlonkParameters<D>, const D: usize>( // Definition may change
    builder: &mut CircuitBuilder<L, D>,
    accumulator: BLSPubkeyVariable,
    attestation: BLSPubkeyVariable, 
) -> BLSPubkeyVariable{

    // Apply BLS signature and return

    // todo!();
    accumulator
}

fn init_bls<L: PlonkParameters<D>, const D: usize>( // Definition may change
    builder: &mut CircuitBuilder<L, D>,
    message: Bytes32Variable, // source + target
    privateKey: BLSPubkeyVariable
) -> BLSPubkeyVariable{

    // privateKey.hash_to_signature()

    todo!();

}
