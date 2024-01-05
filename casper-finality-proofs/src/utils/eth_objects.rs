use curta::maybe_rayon::rayon::str::Bytes;
use plonky2x::{
    frontend::{
        eth::{
            beacon::vars::BeaconValidatorVariable,
            vars::BLSPubkeyVariable
        },
        hash::poseidon::poseidon256::PoseidonHashOutVariable, vars::BytesVariable
    },
    prelude::{
        CircuitBuilder, Variable, BoolVariable, U64Variable, Bytes32Variable, ArrayVariable, CircuitVariable, RichField
    },
    backend::circuit::{DefaultParameters, PlonkParameters},
};

use crate::constants::{
    VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_ROOT_PROOF_LEN, STATE_ROOT_PROOF_LEN, VERSION_OBJ_BYTES
};

#[derive(Debug, Clone, CircuitVariable)]
pub struct ValidatorData { 
    pub is_trusted_validator: BoolVariable,

    validator_index: U64Variable, //TODO: Why is this here? TODO: PoseidonHashOutIndex
    pub beacon_validator_variable: BeaconValidatorVariable,

    pub validator_state_root: Bytes32Variable, // TODO: PoseidonHashOutVariable
    pub validator_leaf: Bytes32Variable, //TODO: PoseidonHashOutVariable
    pub validator_root_proof: ArrayVariable<Bytes32Variable,VALIDATORS_HASH_TREE_DEPTH>, //TODO: PoseidonHashOutVariable
    pub validator_gindex: U64Variable,
}

impl ValidatorData {
    pub fn new(
        is_trusted_validator: BoolVariable,
        validator_index: U64Variable,
        beacon_validator_variable: BeaconValidatorVariable,
        validator_state_root: Bytes32Variable, 
        validator_leaf: Bytes32Variable, 
        validator_root_proof: ArrayVariable<Bytes32Variable,VALIDATORS_HASH_TREE_DEPTH>,  
        validator_gindex: U64Variable,
    ) -> Self {
        ValidatorData {
            is_trusted_validator: is_trusted_validator,
            validator_index: validator_index,
            beacon_validator_variable: beacon_validator_variable,
            validator_state_root: validator_state_root,
            validator_leaf: validator_leaf,
            validator_root_proof: validator_root_proof,
            validator_gindex: validator_gindex, 
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self{ 
        ValidatorData::new(
            builder.read::<BoolVariable>(),
            builder.read::<U64Variable>(),
            builder.read::<BeaconValidatorVariable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>>(),
            builder.read::<U64Variable>(),
        )
    }
}

#[derive(Debug, Clone, CircuitVariable)]
pub struct AttestationData {
    slot: U64Variable,
    index: U64Variable, 

    // LMD GHOST vote
    beacon_block_root: Bytes32Variable,

    // FFG vote
    source: Bytes32Variable,
    target: Bytes32Variable,
}

impl AttestationData {
    pub fn new(
        slot: U64Variable,
        index: U64Variable,
        beacon_block_root: Bytes32Variable,
        source: Bytes32Variable,
        target: Bytes32Variable,
    ) -> Self {
        AttestationData {
            slot: slot,
            index: index,
            beacon_block_root: beacon_block_root,
            source: source,
            target: target,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        AttestationData::new(
            builder.read::<U64Variable>(), 
            builder.read::<U64Variable>(), 
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Fork {
    previous_version: BytesVariable<VERSION_OBJ_BYTES>,
    current_version: BytesVariable<VERSION_OBJ_BYTES>,
    epoch: U64Variable
}

impl Fork {
    pub fn new(
        previous_version: BytesVariable<VERSION_OBJ_BYTES>,
        current_version: BytesVariable<VERSION_OBJ_BYTES>,
        epoch: U64Variable
    ) -> Self {
        Fork {
            previous_version: previous_version,
            current_version: current_version,
            epoch: epoch
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        Fork::new(
            builder.read::<BytesVariable<VERSION_OBJ_BYTES>>(),
            builder.read::<BytesVariable<VERSION_OBJ_BYTES>>(),
            builder.read::<U64Variable>()
        )
    }
}

/*
[NOTE]
    `validators` and `validators_list_proof` are outside of the Attestation class, since
    for each attestation the validator set remains constant
 */ 
#[derive(Debug, Clone)]
pub struct Attestation {
    // Standard attestation data
    data: AttestationData,
    // pub signature: BLSPubkeyVariable, //TODO: BLSVariable 

    // Needed to compute the `signing_root` and verify the `signature`
    fork: Fork,
    genesis_validators_root: Bytes32Variable,
    /*
    We should be able to prove that the majority of validators
    participating in this attestation are part of the validator set
    associated with the state of the last trusted block.
    */
    pub state_root: Bytes32Variable,
    pub state_root_proof: ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>,

    pub validators_root: Bytes32Variable,
    pub validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>,

    // validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
}

impl Attestation {
    pub fn new(
        data: AttestationData,
        // signature: BLSPubkeyVariable,
        fork: Fork,
        genesis_validators_root: Bytes32Variable,
        state_root: Bytes32Variable,
        state_root_proof: ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>,
        validators_root: Bytes32Variable,
        validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>,
        // validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
        // validator_list_proof: ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>,
    ) -> Self {
        Attestation {
            data,
            // signature,
            fork,
            genesis_validators_root,
            state_root,
            state_root_proof,
            validators_root,
            validators_root_proof,
            // validators,
            // validator_list_proof,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        Attestation::new(
            AttestationData::circuit_input(builder),
            // builder.read::<BLSPubkeyVariable>(), //TODO: 
            Fork::circuit_input(builder),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(),

            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(),

            // builder.read::<ArrayVariable<BeaconValidatorVariable,VALIDATORS_PER_COMMITTEE>>(),
        )
    }
}
