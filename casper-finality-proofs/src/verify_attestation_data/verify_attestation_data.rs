use curta::chip::field;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable}, frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable}, vars::{Bytes32Variable, U256Variable}, uint::uint64::U64Variable, hash::poseidon::poseidon256::PoseidonHashOutVariable},
};
use super::super::combine_finality_votes::count_unique_pubkeys::ssz_verify_proof_poseidon;
const VALIDATORS_PER_COMMITTEE: usize = 2048; //2048;
const VALIDATOR_HASH_TREE_DEPTH: usize = 11;
const PLACEHOLDER: usize = 11;

#[derive(Debug, Clone)]
pub struct VerifyAttestationData;


// XValidator = object
// validator_index: ValidatorIndex
// pubkey: ValidatorPubKey
// withdrawal_credentials: Eth2Digest
// effective_balance: Gwei
// slashed: bool
// activation_eligibility_epoch: Epoch
// activation_epoch: Epoch
// exit_epoch: Epoch
// withdrawable_epoch: Epoch
// validator_list_proof: seq[string]
// 

#[derive(Debug, Clone)]
struct ValidatorHashData { 
    validator_index: U64Variable,

    // Equivalent to `validator_list_proof` in ref?
    is_trusted_validator: BoolVariable,
    
    validator_state_root: PoseidonHashOutVariable,
    validator_leaf: PoseidonHashOutVariable,
    validator_branch: ArrayVariable<PoseidonHashOutVariable,VALIDATOR_HASH_TREE_DEPTH>,
    validator_gindex: U64Variable,
}
#[derive(Debug, Clone)]
struct Fork {
    previous_version: Bytes32Variable,
    current_version: Bytes32Variable,
    epoch: Variable
}

// Used as message in BLS Signature
#[derive(Debug, Clone)]
struct AttestationData {
    slot: U256Variable,
    index: U256Variable,

    // LMD GHOST vote
    beacon_block_root: Bytes32Variable,

    // FFG vote
    source: Bytes32Variable,
    target: Bytes32Variable,

}


#[derive(Debug, Clone)]
struct Attestation {
    // Standard attestation data
    data: AttestationData,
    signature: BLSPubkeyVariable,

    // Needed to compute the `signing_root` and verify the `signature`
    fork: Fork,
    genesis_validators_root: Bytes32Variable,

    /*
    We should be able to prove that the majority of validators
    participating in this attestation are part of the validator set
    associated with the state of the last trusted block.
    */
    state_root: Bytes32Variable,
    state_root_proof: ArrayVariable<Bytes32Variable, PLACEHOLDER>,

    validators_root: Bytes32Variable,
    validators_root_proof: ArrayVariable<Bytes32Variable, PLACEHOLDER>,

    validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
}


impl Circuit for VerifyAttestationData {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
   
        //TODO: 1. Sigma <- Challange Assrt != 0

        let prev_block_root = builder.read::<Bytes32Variable>();
        let validators = builder.read::<ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>>();

        // Read ValidatorHashData
        let mut validator_hash_vec: Vec<ValidatorHashData> = Vec::new();
        for _ in 0..VALIDATORS_PER_COMMITTEE {
            
            validator_hash_vec.push( ValidatorHashData {
                validator_state_root: builder.read::<PoseidonHashOutVariable>(),
                is_trusted_validator: builder.read::<BoolVariable>(),
                validator_leaf: builder.read::<PoseidonHashOutVariable>(),
                validator_branch: builder.read::<ArrayVariable<PoseidonHashOutVariable, VALIDATOR_HASH_TREE_DEPTH>>(),
                validator_gindex: builder.read::<U64Variable>(),
                validator_index: builder.read::<U64Variable>()
            })
        }

        // Read Attestation
        let attestation =  Attestation {
            data: AttestationData {
                slot: builder.read::<U256Variable>(),
                index: builder.read::<U256Variable>(), 
                beacon_block_root: builder.read::<Bytes32Variable>(),
                source: builder.read::<Bytes32Variable>(),
                target: builder.read::<Bytes32Variable>(),
            },
            signature: builder.read::<BLSPubkeyVariable>(),

            fork: Fork {
                previous_version: builder.read::<Bytes32Variable>(),
                current_version: builder.read::<Bytes32Variable>(),
                epoch: builder.read::<Variable>()
            },
            genesis_validators_root: builder.read::<Bytes32Variable>(),

            state_root: builder.read::<Bytes32Variable>(),
            state_root_proof: builder.read::<ArrayVariable<Bytes32Variable, PLACEHOLDER>>(),

            validators_root: builder.read::<Bytes32Variable>(),
            validators_root_proof: builder.read::<ArrayVariable<Bytes32Variable, PLACEHOLDER>>(),

            validators: builder.read::<ArrayVariable<BeaconValidatorVariable,VALIDATORS_PER_COMMITTEE>>(),
        };

        // 2. 3.
        block_merkle_branch_proof(
            builder,
            prev_block_root,
            attestation.clone()
        );

        
        let mut pk_accumulator = validators[0].pubkey;
        for i in 1..VALIDATORS_PER_COMMITTEE {

            // 4. Accumulate BLS Signature
            pk_accumulator = accumulate_bls(builder,pk_accumulator, validators[i].pubkey);

            // 5. Verify Validator set
            verify_validator(builder, validators[i], validator_hash_vec[i].clone());
        }

        //Assert that BLS Signature is correct
        builder.assert_is_equal(attestation.signature, pk_accumulator);

        // Private BLS Accumulator for the recurssive proof
        let zero_bls = validators[0].pubkey;
        let mut private_accumulator = validators[0].pubkey; // TODO: validator hash

        for i in 1..VALIDATORS_PER_COMMITTEE {
                let value_to_add = builder.select(
                    validator_hash_vec[i].is_trusted_validator,
                    validators[i].pubkey, // TODO: validator hash
                    zero_bls
                ); 
                accumulate_bls(builder,private_accumulator, value_to_add); // TODO: validator hash
        }

        //Will accumulate sorted validator index hash messages 
    }
}


fn block_merkle_branch_proof<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    prev_block_root: Bytes32Variable,
    attestation: Attestation
) {
    let field_eleven = <L as PlonkParameters<D>>::Field::from_canonical_u64(11);
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
    validator: BeaconValidatorVariable,
    validator_hash: ValidatorHashData

) 
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    
    // Ordering check
    let align_epoch1 = builder.lte(validator.activation_eligibility_epoch, validator.activation_epoch);
    let align_epoch2 = builder.lte(validator.activation_epoch, validator.withdrawable_epoch);
    let align_epoch3 = builder.lte(validator.withdrawable_epoch, validator.exit_epoch);

    let valid_epochs = vec![align_epoch1.variable,align_epoch2.variable, align_epoch3.variable];
    let aligned_count = builder.add_many(&valid_epochs);
    // implement add_many for BoolVariable or better yet and_many

    let field_three = <L as PlonkParameters<D>>::Field::from_canonical_u64(3);
    let const_three = builder.constant(field_three);
    builder.assert_is_equal(aligned_count, const_three);

    // Prove validator is part of the validator set

    //TODO: BeaconValidatorVariable and ValidatorHashData should be the same object

    ssz_verify_proof_poseidon(
        builder,
        validator_hash.validator_state_root,
        validator_hash.validator_leaf,
        validator_hash.validator_branch.as_slice(),
        validator_hash.validator_gindex
    );
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
