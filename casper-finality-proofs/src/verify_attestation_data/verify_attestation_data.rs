use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters}, 
    frontend::{eth::vars::BLSPubkeyVariable, vars::{Bytes32Variable,Variable, SSZVariable, ByteVariable, BytesVariable}, 
    uint::uint64::U64Variable, 
    hash::poseidon::poseidon256::PoseidonHashOutVariable},
};

use crate::{utils::eth_objects::{ValidatorData, Fork, AttestationData, Attestation, BeaconValidatorVariable}, constants::{TEST_VALIDATORS_IN_COMMITMENT_SIZE, VALIDATOR_ROOT_GINDEX, STATE_ROOT_GINDEX}};
use crate::constants::{VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_PER_COMMITTEE,VALIDATORS_ROOT_PROOF_LEN, STATE_ROOT_PROOF_LEN};
use crate::combine_finality_votes::count_unique_pubkeys::ssz_verify_proof_poseidon;

#[derive(Debug, Clone)]
pub struct VerifyAttestationData;

impl Circuit for VerifyAttestationData {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {

        //TODO: Sigma <-- Challange
        let sigma: U64Variable = builder.one();
        let _false: BoolVariable = builder._false();

        //TODO: Sigma != 0
        // let is_zero_sigma_pred = builder.is_zero(sigma);
        // builder.assert_is_equal(is_zero_sigma_pred, _false);

        let prev_block_root = builder.read::<Bytes32Variable>(); 

        let attestation = Attestation::circuit_input(builder);

        //TODO: This should be part of the final proof as it is only needed once
        // 2. 3.
        block_merkle_branch_proof(
            builder,
            prev_block_root,
            attestation.clone()
        );

        let first_validator = builder.read::<ValidatorData>();
        // Private Accumulator 
        let mut private_accumulator = builder.zero();
        private_accumulator = accumulate_private(
            builder,
            private_accumulator,
            first_validator.validator_index,
            sigma,
        );

        //BLS Accumulator
        let mut bls_accumulator = first_validator.beacon_validator_variable.pubkey;
        bls_accumulator = accumulate_bls(builder,bls_accumulator, first_validator.beacon_validator_variable.pubkey);
        
        verify_validator(builder, first_validator.clone(), attestation.validators_root);

        for _ in 1..TEST_VALIDATORS_IN_COMMITMENT_SIZE {
            let cur_validator = builder.read::<ValidatorData>();
            verify_validator(builder, cur_validator.clone(), attestation.validators_root);
            
            private_accumulator = accumulate_private(
                builder,
                private_accumulator,
                first_validator.validator_index,
                sigma,
            );

            bls_accumulator = accumulate_bls(builder,bls_accumulator, first_validator.beacon_validator_variable.pubkey);
        }

        //TODO: 
        // builder.assert_is_equal(attestation.signature, bls_accumulator);

        // // Private BLS Accumulator for the recurssive proof
        // let mut pk_accumulator = validator_vec[0].beacon_validator_variable.pubkey;
        // for i in 1..VALIDATORS_PER_COMMITTEE {

        //     // 4. Accumulate BLS Signature
        //     pk_accumulator = accumulate_bls(builder,pk_accumulator, validator_vec[i].beacon_validator_variable.pubkey);
               //TODO: Verify that BLS checks out

        //     // 5. Verify Validator set
        //     verify_validator(builder, validator_vec[i].clone());
        // }

        // // Add validator pubkey to commitment if validator is trusted
        // for i in 1..VALIDATORS_PER_COMMITTEE { //TODO: (token + element*sigma) % MODULUS
        //         let value_to_add = builder.select(
        //             validator_vec[i].is_trusted_validator,
        //             validator_vec[i].beacon_validator_variable.pubkey, // TODO: validator hash
        //             zero_bls
        //         ); 
        //         accumulate_bls(builder,private_accumulator, value_to_add); // TODO: validator hash
        // }
        builder.write(attestation.data.source);
        builder.write(attestation.data.target);
        builder.write(private_accumulator);
        builder.write(sigma); // Ingested by CombineFinalityVotes2
    }
}


fn accumulate_private<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    private_accumulator: U64Variable,
    validator_index: U64Variable,
    sigma: U64Variable,
    
) -> U64Variable {
    let multiplied = builder.mul(validator_index,sigma);
    builder.add(private_accumulator, multiplied)
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

fn verify_validator<L: PlonkParameters<D>, const D: usize>( // TODO: Should pass only trusted_validators
    builder: &mut CircuitBuilder<L, D>,
    validator: ValidatorData,
    validators_root: Bytes32Variable
) 
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    
    // Ordering check TODO: Maybe not needed?
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

    let validator_leaf = hash_validator(builder, validator.beacon_validator_variable);

    let first_validators_gindex: U64Variable = builder.constant(2u64.pow(VALIDATORS_HASH_TREE_DEPTH as u32));
    let gindex = builder.add(first_validators_gindex, validator.validator_index);

    builder.ssz_verify_proof(
        validators_root,
        validator_leaf,
        validator.validator_root_proof.as_slice(),
        gindex,
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

/*
    # Python reference implementation
    digest(
            digest(
                digest(digest(pubkey)              , withdrawal_credentials),
                digest(effective_balance           , slashed),
            ),
            digest(
                digest(activation_eligibility_epoch, activation_epoch),
                digest(exit_epoch                  , withdrawable_epoch),
            ),
        )
*/
pub fn hash_validator<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>, validator: BeaconValidatorVariable) -> Bytes32Variable {

    let zero: BoolVariable = builder._false();

    let mut pubkey_bytes: Vec<ByteVariable> = Vec::new();
    pubkey_bytes.extend(&validator.pubkey.0.0);
    pubkey_bytes.extend(&[builder.zero::<ByteVariable>(); 16]);
    let pubkey_bytes32 = builder.curta_sha256(&pubkey_bytes);

    let effective_balance_bytes32 = builder.ssz_hash_tree_root(validator.effective_balance);

    let mut slashed_bytes32 = Bytes32Variable(BytesVariable::<32>(vec![builder.zero::<ByteVariable>();32].try_into().unwrap()));
    let slashed_byte_variable = ByteVariable([validator.slashed, zero, zero, zero, zero, zero, zero, zero]);
    slashed_bytes32.0.0[0] = slashed_byte_variable;
    
    let activation_eligibility_epoch_bytes32 = builder.ssz_hash_tree_root(validator.activation_eligibility_epoch);
    let activation_epoch_bytes32 = builder.ssz_hash_tree_root(validator.activation_epoch);
    let exit_epoch_bytes32 = builder.ssz_hash_tree_root(validator.exit_epoch);
    let withdrawable_epoch_bytes32 = builder.ssz_hash_tree_root(validator.withdrawable_epoch);

    let digest_pk_wc = builder.curta_sha256_pair(pubkey_bytes32, validator.withdrawal_credentials);
    let digest_eb_sl = builder.curta_sha256_pair(effective_balance_bytes32, slashed_bytes32);
    let digest_comp1 = builder.curta_sha256_pair(digest_pk_wc, digest_eb_sl);

    let digest_aee_ae: Bytes32Variable = builder.curta_sha256_pair(activation_eligibility_epoch_bytes32, activation_epoch_bytes32);
    let digest_ee_we = builder.curta_sha256_pair(exit_epoch_bytes32, withdrawable_epoch_bytes32);
    let digest_comp2 = builder.curta_sha256_pair(digest_aee_ae, digest_ee_we);

    builder.curta_sha256_pair(digest_comp1, digest_comp2)
}

fn accumulate_bls<L: PlonkParameters<D>, const D: usize>( //TODO:
    builder: &mut CircuitBuilder<L, D>,
    accumulator: BLSPubkeyVariable,
    pubkey: BLSPubkeyVariable, 
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

