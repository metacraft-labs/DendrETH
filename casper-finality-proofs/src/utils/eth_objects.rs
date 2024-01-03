use plonky2x::{
    frontend::{
        eth::{
            beacon::vars::BeaconValidatorVariable,
            vars::BLSPubkeyVariable
        },
        hash::poseidon::poseidon256::PoseidonHashOutVariable
    },
    prelude::{
        CircuitBuilder, Variable, BoolVariable, U256Variable, U64Variable, Bytes32Variable, ArrayVariable
    },
    backend::circuit::{DefaultParameters, PlonkParameters},
};

use super::super::constants::{
    VALIDATORS_PER_COMMITTEE, VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_ROOT_PROOF_LEN, STATE_ROOT_PROOF_LEN
};

type L = DefaultParameters;
const D: usize = 2;

#[derive(Debug, Clone)]
pub struct ValidatorHashData { 
    validator_index: U64Variable,

    // Equivalent to `validator_list_proof` in ref?
    pub is_trusted_validator: BoolVariable,
    
    pub validator_state_root: PoseidonHashOutVariable,
    pub validator_leaf: PoseidonHashOutVariable,
    pub validator_branch: ArrayVariable<PoseidonHashOutVariable,VALIDATORS_HASH_TREE_DEPTH>,
    pub validator_gindex: U64Variable,
}

impl ValidatorHashData {
    pub fn new(
        validator_index: U64Variable,
        is_trusted_validator: BoolVariable,
        validator_state_root: PoseidonHashOutVariable,
        validator_leaf: PoseidonHashOutVariable,
        validator_branch: ArrayVariable<PoseidonHashOutVariable,VALIDATORS_HASH_TREE_DEPTH>,
        validator_gindex: U64Variable,
    ) -> Self {
        ValidatorHashData {
            validator_index: validator_index,
            is_trusted_validator: is_trusted_validator,
            validator_state_root: validator_state_root,
            validator_leaf: validator_leaf,
            validator_branch: validator_branch,
            validator_gindex: validator_gindex,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self{ 
        ValidatorHashData::new(
            builder.read::<U64Variable>(), //TODO: PoseidonHashOutIndex
            builder.read::<BoolVariable>(),
            builder.read::<PoseidonHashOutVariable>(),
            builder.read::<PoseidonHashOutVariable>(),
            builder.read::<ArrayVariable<PoseidonHashOutVariable, VALIDATORS_HASH_TREE_DEPTH>>(),
            builder.read::<U64Variable>(),
        )
    }

}

#[derive(Debug, Clone)]
pub struct AttestationData {
    slot: U256Variable,
    index: U256Variable,

    // LMD GHOST vote
    beacon_block_root: Bytes32Variable,

    // FFG vote
    source: Bytes32Variable,
    target: Bytes32Variable,

}

impl AttestationData {
    pub fn new(
        slot: U256Variable,
        index: U256Variable,
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
            builder.read::<U256Variable>(),
            builder.read::<U256Variable>(), 
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Fork {
    previous_version: Bytes32Variable,
    current_version: Bytes32Variable,
    epoch: Variable
}

impl Fork {
    pub fn new(
        previous_version: Bytes32Variable,
        current_version: Bytes32Variable,
        epoch: Variable
    ) -> Self {
        Fork {
            previous_version: previous_version,
            current_version: current_version,
            epoch: epoch
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        Fork::new(
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<Variable>()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Attestation {
    // Standard attestation data
    data: AttestationData,
    pub signature: BLSPubkeyVariable,

    // Needed to compute the `signing_root` and verify the `signature`
    fork: Fork,
    genesis_validators_root: Bytes32Variable,
    /*
    We should be able to prove that the majority of validators
    participating in this attestation are part of the validator set
    associated with the state of the last trusted block.
    */
    pub state_root: Bytes32Variable,
    pub state_root_proof: ArrayVariable<Bytes32Variable, 3>,

    pub validators_root: Bytes32Variable,
    pub validators_root_proof: ArrayVariable<Bytes32Variable, 5>,

    validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,

    //TODO: Include
    // validator_list_proof: ArrayVariable<Bytes32Variable, VALIDATOR_HASH_TREE_DEPTH>
}

impl Attestation {
    pub fn new(
        data: AttestationData,
        signature: BLSPubkeyVariable,
        fork: Fork,
        genesis_validators_root: Bytes32Variable,
        state_root: Bytes32Variable,
        state_root_proof: ArrayVariable<Bytes32Variable, 3>,
        validators_root: Bytes32Variable,
        validators_root_proof: ArrayVariable<Bytes32Variable, 5>,
        validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
        // validator_list_proof: ArrayVariable<Bytes32Variable, 41>,
    ) -> Self {
        Attestation {
            data,
            signature,
            fork,
            genesis_validators_root,
            state_root,
            state_root_proof,
            validators_root,
            validators_root_proof,
            validators,
            // validator_list_proof,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        Attestation::new(
            AttestationData::circuit_input(builder),
            builder.read::<BLSPubkeyVariable>(),
            Fork::circuit_input(builder),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(),

            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(),

            builder.read::<ArrayVariable<BeaconValidatorVariable,VALIDATORS_PER_COMMITTEE>>(),
        )
    }
}
