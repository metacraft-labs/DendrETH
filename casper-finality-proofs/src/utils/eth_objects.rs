use plonky2::{field::{goldilocks_field::GoldilocksField, types::Field}, hash::hash_types::HashOut};
use plonky2x::{
    backend::circuit::{PlonkParameters, PublicInput}, frontend::{
        eth::vars::BLSPubkeyVariable, hash::poseidon::poseidon256::PoseidonHashOutVariable, vars::BytesVariable
    }, prelude::{
        ArrayVariable, BoolVariable, Bytes32Variable, CircuitBuilder, CircuitVariable, RichField, U64Variable, Variable
    }, utils::{bytes, bytes32}
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{constants::{
    STATE_ROOT_PROOF_LEN, VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_ROOT_PROOF_LEN, VERSION_OBJ_BYTES, ZERO_HASHES
}, weigh_justification_and_finalization::justification_bits::JustificationBitsVariable};

fn deserialize_validator_list_proof<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let list_proof: Vec<String> = Deserialize::deserialize(deserializer)?;

    let padded_result: Vec<String> = list_proof.into_iter()
        .enumerate()
        .map(|(index, item)|
            if item.is_empty() {
                 ZERO_HASHES[index].to_string()
                } 
            else { item.to_string() })
        .collect();

    Ok(padded_result)
}
    
fn u64_to_bool_vec(value: u64) -> Vec<bool> {
    let mut result = Vec::with_capacity(4);

    // Extracting bits using bitwise operations
    for i in (0..4).rev() {
        let bit = (value >> i) & 1 == 1;
        result.push(bit);
    }

    result
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeaconStateInput {
    pub justification_bits: u64,
    pub previous_justified_checkpoint: CheckpointInput,
    pub current_justified_checkpoint: CheckpointInput,
}

impl BeaconStateInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>){
        // input.write::<JustificationBitsVariable>(
            // u64_to_bool_vec(self.justification_bits)
                // .iter()
                // .collect()
            // );

        input.write::<ArrayVariable<BoolVariable, 4>>(u64_to_bool_vec(self.justification_bits));
        self.previous_justified_checkpoint.write(&mut input);
        self.current_justified_checkpoint.write(&mut input);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeaconValidatorInput {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
    pub withdrawable_epoch: u64,
}

impl BeaconValidatorInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>){
        input.write::<BLSPubkeyVariable>(bytes!(self.pubkey.clone())); 
        input.write::<Bytes32Variable>(bytes32!(self.withdrawal_credentials)); 
        input.write::<U64Variable>(self.effective_balance);
        input.write::<BoolVariable>(self.slashed);
        input.write::<U64Variable>(self.activation_eligibility_epoch); 
        input.write::<U64Variable>(self.activation_epoch); 
        input.write::<U64Variable>(self.exit_epoch); 
        input.write::<U64Variable>(self.withdrawable_epoch); 
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HashOutPoseidonInput {
    pub elements: Vec<u64>,
}

impl HashOutPoseidonInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<PoseidonHashOutVariable>(HashOut::from_vec(
            self.elements
            .iter()
            .map(|element| <L as PlonkParameters<D>>::Field::from_canonical_u64(*element))
            .collect()
        ));
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataPoseidonInput {
    trusted: bool,
    validator_index: u64,

    #[serde(flatten)]
    beacon_validator_variable: BeaconValidatorInput,

    validator_poseidon_hash: HashOutPoseidonInput,
    validator_poseidon_proof: Vec<HashOutPoseidonInput>,
}

impl ValidatorDataPoseidonInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        input.write::<BoolVariable>(self.trusted); 
        input.write::<U64Variable>(self.validator_index); 
        self.beacon_validator_variable.write(&mut input);

        self.validator_poseidon_hash.write(input);
        for _ in 0..VALIDATORS_HASH_TREE_DEPTH {
            self.validator_poseidon_proof[0].write(input); //TODO: Smarter way to do this?
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataInput {
    trusted: bool,
    validator_index: u64,

    #[serde(flatten)]
    beacon_validator_variable: BeaconValidatorInput,

    #[serde(deserialize_with="deserialize_validator_list_proof")]
    validator_list_proof: Vec<String>,
}

impl ValidatorDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        input.write::<BoolVariable>(self.trusted); 
        input.write::<U64Variable>(self.validator_index); 
        self.beacon_validator_variable.write(&mut input);

        input.write::<ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>>(
            self.validator_list_proof
            .iter()
            .map(|element| bytes32!(element))
            .collect()
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInput {
    root: String,
    epoch: u64,
}

impl CheckpointInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<Bytes32Variable>(bytes32!(self.root));
        input.write::<U64Variable>(self.epoch);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInput {
    previous_version: String,
    current_version: String,
    epoch: u64,
}

impl ForkInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<BytesVariable<VERSION_OBJ_BYTES>>(bytes!(&self.previous_version));
        input.write::<BytesVariable<VERSION_OBJ_BYTES>>(bytes!(&self.current_version));
        input.write::<U64Variable>(self.epoch); 
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationDataInput {
    slot: u64,
    index: u64,

    beacon_block_root: String,

    // #[serde(deserialize_with="deserialize_checkpoint")]
    source: CheckpointInput,

    // #[serde(deserialize_with="deserialize_checkpoint")]
    target: CheckpointInput,
}


impl AttestationDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        input.write::<U64Variable>(self.slot); 
        input.write::<U64Variable>(self.index);
        input.write::<Bytes32Variable>(bytes32!(self.beacon_block_root));
        self.source.write(&mut input);
        self.target.write(&mut input);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationInput {
    data: AttestationDataInput,
    // signature: String,
    fork: ForkInput,
    genesis_validators_root: String,
    state_root: String,
    state_root_proof: Vec<String>,
    validators_root: String,
    validators_root_proof: Vec<String>,
}

impl AttestationInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        self.data.write(&mut input);

        // input.write::<BLSPubkeyVariable>(); //TODO:
        self.fork.write(&mut input);
        input.write::<Bytes32Variable>(bytes32!(self.genesis_validators_root));

        input.write::<Bytes32Variable>(bytes32!(self.state_root));
        input.write::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(
            self.state_root_proof
                .iter()
                .map(|element| bytes32!(element))
                .collect(),
        );

        input.write::<Bytes32Variable>(bytes32!(self.validators_root));
        input.write::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(
            self.validators_root_proof
                .iter()
                .map(|element| bytes32!(element))
                .collect(),
        );
    }
}


#[derive(Debug, Clone, Copy, CircuitVariable)]
pub struct BeaconValidatorVariable {
    pub pubkey: BLSPubkeyVariable,
    pub withdrawal_credentials: Bytes32Variable,
    pub effective_balance: U64Variable,
    pub slashed: BoolVariable,
    pub activation_eligibility_epoch: U64Variable,
    pub activation_epoch: U64Variable,
    pub exit_epoch: U64Variable,
    pub withdrawable_epoch: U64Variable,
}

impl BeaconValidatorVariable {
    pub fn new(
        pubkey: BLSPubkeyVariable,
        withdrawal_credentials: Bytes32Variable,
        effective_balance: U64Variable,
        slashed: BoolVariable,
        activation_eligibility_epoch: U64Variable,
        activation_epoch: U64Variable,
        exit_epoch: U64Variable,
        withdrawable_epoch: U64Variable,
    ) -> Self {
        BeaconValidatorVariable {
            pubkey: pubkey,
            withdrawal_credentials: withdrawal_credentials,
            effective_balance: effective_balance,
            slashed: slashed,
            activation_eligibility_epoch: activation_eligibility_epoch,
            activation_epoch: activation_epoch,
            exit_epoch: exit_epoch,
            withdrawable_epoch: withdrawable_epoch,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self{ 
        BeaconValidatorVariable::new(
            builder.read::<BLSPubkeyVariable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<U64Variable>(),
            builder.read::<BoolVariable>(),
            builder.read::<U64Variable>(),
            builder.read::<U64Variable>(),
            builder.read::<U64Variable>(),
            builder.read::<U64Variable>()
        )
    }
}

#[derive(Debug, Clone, CircuitVariable)]
pub struct ValidatorDataPoseidon { 
    pub trusted: BoolVariable,
    
    pub validator_gindex: U64Variable,
    pub beacon_validator_variable: BeaconValidatorVariable,
    pub beacon_validator_variable_hash: PoseidonHashOutVariable,

    pub validator_root_proof: ArrayVariable<PoseidonHashOutVariable ,VALIDATORS_HASH_TREE_DEPTH>,
}

impl ValidatorDataPoseidon {
    pub fn new(
        trusted: BoolVariable,
        validator_gindex: U64Variable,
        beacon_validator_variable: BeaconValidatorVariable,
        beacon_validator_variable_hash: PoseidonHashOutVariable,
        validator_root_proof: ArrayVariable<PoseidonHashOutVariable,VALIDATORS_HASH_TREE_DEPTH>,  

    ) -> Self {
        ValidatorDataPoseidon {
            trusted: trusted,
            validator_gindex: validator_gindex,
            beacon_validator_variable: beacon_validator_variable,
            beacon_validator_variable_hash: beacon_validator_variable_hash,
            validator_root_proof: validator_root_proof,
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self{ 
        ValidatorDataPoseidon::new(
            builder.read::<BoolVariable>(),
            builder.read::<U64Variable>(),
            builder.read::<BeaconValidatorVariable>(),
            builder.read::<PoseidonHashOutVariable>(),
            builder.read::<ArrayVariable<PoseidonHashOutVariable, VALIDATORS_HASH_TREE_DEPTH>>(),
        )
    }
}


#[derive(Debug, Clone, CircuitVariable)]
pub struct ValidatorData { 
    pub trusted: BoolVariable,
    
    pub validator_index: U64Variable, // validator_gindex
    pub beacon_validator_variable: BeaconValidatorVariable,

    // pub validator_state_root: Bytes32Variable, // TODO: PoseidonHashOutVariable
    // pub validator_leaf: Bytes32Variable, //TODO: PoseidonHashOutVariable
    pub validator_root_proof: ArrayVariable<Bytes32Variable,VALIDATORS_HASH_TREE_DEPTH>, //TODO: PoseidonHashOutVariable
}

impl ValidatorData {
    pub fn new(
        trusted: BoolVariable,
        validator_index: U64Variable,

        beacon_validator_variable: BeaconValidatorVariable,
        // validator_state_root: Bytes32Variable, 
        // validator_leaf: Bytes32Variable, 
        validator_root_proof: ArrayVariable<Bytes32Variable,VALIDATORS_HASH_TREE_DEPTH>,  
        // validator_gindex: U64Variable,
    ) -> Self {
        ValidatorData {
            trusted: trusted,
            validator_index: validator_index,
            beacon_validator_variable: beacon_validator_variable,
            // validator_state_root: validator_state_root,
            // validator_leaf: validator_leaf,
            validator_root_proof: validator_root_proof,
            // validator_gindex: validator_gindex, 
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self{ 
        ValidatorData::new(
            builder.read::<BoolVariable>(),
            builder.read::<U64Variable>(),
            builder.read::<BeaconValidatorVariable>(),
            // builder.read::<Bytes32Variable>(),
            // builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>>(),
            // builder.read::<U64Variable>(),
        )
    }
}

#[derive(Debug, Copy, Clone, CircuitVariable)]
pub struct CheckpointVariable {
    pub root: Bytes32Variable,
    pub epoch: U64Variable,
}

#[derive(Debug, Clone, CircuitVariable)]
pub struct AttestationData {
    pub slot: U64Variable,
    index: U64Variable, 

    // LMD GHOST vote
    beacon_block_root: Bytes32Variable,

    // FFG vote
    pub source: CheckpointVariable,
    pub target: CheckpointVariable,
}

impl AttestationData {
    pub fn new(
        slot: U64Variable,
        index: U64Variable,
        beacon_block_root: Bytes32Variable,
        source: CheckpointVariable,
        target: CheckpointVariable,
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
            builder.read::<CheckpointVariable>(),
            builder.read::<CheckpointVariable>(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BeaconState {
    pub justification_bits: JustificationBitsVariable,
    pub previous_justified_checkpoint: CheckpointVariable,
    pub current_justified_checkpoint: CheckpointVariable,
}

impl BeaconState {
    pub fn new(
        justification_bits: JustificationBitsVariable,
        previous_justified_checkpoint: CheckpointVariable,
        current_justified_checkpoint: CheckpointVariable,
    ) -> BeaconState {
        BeaconState {
            justification_bits: justification_bits,
            previous_justified_checkpoint: previous_justified_checkpoint,
            current_justified_checkpoint: current_justified_checkpoint
        }
    }

    pub fn circuit_input<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L,D>) -> Self {
        BeaconState {
            justification_bits: builder.read::<JustificationBitsVariable>(),
            previous_justified_checkpoint: builder.read::<CheckpointVariable>(),
            current_justified_checkpoint: builder.read::<CheckpointVariable>(),
        }
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
pub struct AttestationPoseidon {
    // Standard attestation data
    pub data: AttestationData,
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

    pub validators_root: PoseidonHashOutVariable,
    pub validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>,

    // validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
}

impl AttestationPoseidon {
    pub fn new(
        data: AttestationData,
        // signature: BLSPubkeyVariable,
        fork: Fork,
        genesis_validators_root: Bytes32Variable,
        state_root: Bytes32Variable,
        state_root_proof: ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>,
        validators_root: PoseidonHashOutVariable,
        validators_root_proof: ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>,
        // validators: ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_COMMITTEE>,
        // validator_list_proof: ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>,
    ) -> Self {
        AttestationPoseidon {
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
        AttestationPoseidon::new(
            AttestationData::circuit_input(builder),
            // builder.read::<BLSPubkeyVariable>(), //TODO: 
            Fork::circuit_input(builder),
            builder.read::<Bytes32Variable>(),
            builder.read::<Bytes32Variable>(),
            builder.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(),

            builder.read::<PoseidonHashOutVariable>(),
            builder.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(),

            // builder.read::<ArrayVariable<BeaconValidatorVariable,VALIDATORS_PER_COMMITTEE>>(),
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
    pub data: AttestationData,
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
