use plonky2x::backend::circuit::{Circuit, PlonkParameters};
use plonky2x::backend::function::Plonky2xFunction;
use plonky2x::frontend::eth::beacon::generators::BeaconBalanceBatchWitnessHint;
use plonky2x::frontend::eth::beacon::vars::BeaconValidatorVariable;
use plonky2x::frontend::eth::vars::BLSPubkeyVariable;
use plonky2x::prelude::{CircuitBuilder, Variable, U64Variable, Bytes32Variable, ArrayVariable, CircuitVariable, U256Variable, BoolVariable};
use plonky2x::utils::eth::beacon::{BeaconHeaderContainer, BeaconValidator, BeaconHeader};

use crate::checkpoint::Checkpoint;

// Constants
const JUSTIFICATION_BITS_LENGTH: i32 = 4;
const SLOTS_PER_EPOCH: i32 = 32;
const SUBCOMMITTEE_COUNT: i32 = 64;
const EPOCHS_PER_ETH1_VOTING_PERIOD: i32 = 64;
const SUBCOMMITTEE_SIZE: i32 = 128;
const MAX_ATTESTATIONS: i32 = 128;
const MAX_VALIDATORS_PER_COMMITTEE: i32 = 2048;

#[derive(Debug, Clone)]
struct VerifySubcommitteeVotes;

struct BeaconState {
    genesis_time: uint64,
    genesis_validators_root: Root,
    slot: Slot,
    fork: Fork,
    // History
    latest_block_header: BeaconBlockHeader,
    
    block_roots: ArrayVariable<Bytes32Variable, SLOTS_PER_HISTORICAL_ROOT>,
    state_roots: ArrayVariable<Bytes32Variable, SLOTS_PER_HISTORICAL_ROOT>, 
    historical_roots: ArrayVariable<Bytes32Variable, HISTORICAL_ROOTS_LIMIT>,

    // Eth1
    eth1_data: Eth1Data,
    eth1_data_votes: ArrayVariable<Eth1Data, EPOCHS_PER_ETH1_VOTING_PERIOD * SLOTS_PER_EPOCH>,
    eth1_deposit_index: U64Variable,

    // Registry
    validators: ArrayVariable<BeaconValidatorVariable, VALIDATOR_REGISTRY_LIMIT>,
    balances: ArrayVariable<U256Variable, VALIDATOR_REGISTRY_LIMIT>,

    // Randomness
    randao_mixes: ArrayVariable<Bytes32Variable, EPOCHS_PER_HISTORICAL_VECTOR>,

    // Slashings
    slashings: ArrayVariable<U256Variable, EPOCHS_PER_SLASHINGS_VECTOR>, // Per-epoch sums of slashed effective balances,

    // Attestations
    previous_epoch_attestations: ArrayVariable<PendingAttestation, MAX_ATTESTATIONS * SLOTS_PER_EPOCH>,
    current_epoch_attestations: ArrayVariable<PendingAttestation, MAX_ATTESTATIONS * SLOTS_PER_EPOCH>,

    //  Finality
    justification_bits: ArrayVariable<BoolVariable, JUSTIFICATION_BITS_LENGTH>, // Bit set for every recent justified epoch,
    previous_justified_checkpoint: Checkpoint,  // Previous epoch snapshot,
    current_justified_checkpoint: Checkpoint,
    finalized_checkpoint: Checkpoint,
}

struct PendingAttestation {
    aggregation_bits: ArrayVariable<BoolVariable, MAX_VALIDATORS_PER_COMMITTEE>, // Bitlist[MAX_VALIDATORS_PER_COMMITTEE]
    data: AttestationData,
    inclusion_delay: U256Variable,
    proposer_index: U64Variable,
}

struct Eth1Data {
    deposit_root: Bytes32Variable,
    deposit_count: U64Variable,
    block_hash: Bytes32Variable,
}

struct BeaconBlock {
    attestations: ArrayVariable<Attestation, SUBCOMMITTEE_COUNT>,

}

struct Attestation {
    aggregation_bits: Bytes32Variable, // Bitlist[MAX_VALIDATORS_PER_COMMITTEE]
    data: AttestationData,
    signature: BLSSignature,
}
struct AttestationData {

    slot: U256Variable,
    index: U256Variable,
    
    // LMD GHOST vote
    beacon_block_root: Bytes32Variable, //TODO: Type?

    // FFG vote
    source: Bytes32Variable, //TODO: Type?
    target: Bytes32Variable //TODO: Type?

}

impl Circuit for VerifySubcommitteeVotes {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        
        let state_root = builder.read::<Bytes32Variable>();
        let validators_root = builder.read::<Bytes32Variable>();
        let validators_proof = builder.read::<ArrayVariable<Bytes32Variable, 5>>();
        let validators_gindex = U64Variable::constant(builder, 43); // Is this correct?

        let validator_list_length = builder.read::<Variable>();
        let validator_list = 
            builder.read::<ArrayVariable<BeaconValidatorVariable>,validator_list_length>();

        verify_subcommittee_votes(builder)
    } 

    // TODO: We have to use a poseidon commitment mapper to build the validator hashtree root 
    fn verify_validator<L: PlonkParameters<D>, const D: usize>(
            builder: &mut CircuitBuilder<L, D>,
            validator: BeaconValidatorVariable
        ) { 

            let align_epoch1 = builder.lte(validator.activation_eligibility_epoch, validator.activation_epoch);
            let align_epoch2 = builder.lte(validator.activation_epoch, validator.withdrawable_epoch);
            let align_epoch3 = builder.lte(validator.withdrawable_epoch, validator.exit_epoch);

            let aligned_count = builder.add_many(&[align_epoch1,align_epoch2, align_epoch3]);
            // implement add_many for BoolVariable or better yet and_many

            builder.assert_is_equal(aligned_count, Variable::constant(builder, 3));

            let accumulated_key: BLSPubkeyVariable = pubkey_accumulator(builder, validator);

            accumulated_key

        }

    fn pubkey_accumulator<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        element: BLSPubkeyVariable
    ) -> BLSPubkeyVariable{

        todo!();
    }

    fn attestation_accumulator<L: PlonkParameters<D>, const D: usize>( // Do I need this?
        builder: &mut CircuitBuilder<L, D>,
        attestation: AttestationData // Definition may change
    ) {

        // Apply BLS signature and return

        todo!();
    }

    fn verify_subcommittee_votes<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        state_root: Bytes32Variable,
        validators_root: Bytes32Variable,
        validators_proof: ArrayVariable<Bytes32Variable, 5>,
        validators_gindex: U64Variable
    ) {

        // Prove validator set is part of the state root
        builder.ssz_verify_proof(state_root, validators_root, validators_proof, validators_gindex);

        // build poseidon commitment mapper (for validator in validator set )
        // prove poseidon hash tree root maps to sha256 validator hash tree root

    }

}

