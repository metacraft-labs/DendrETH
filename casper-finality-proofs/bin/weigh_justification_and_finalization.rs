use casper_finality_proofs::{
    constants::{
        BEACON_STATE_BLOCK_ROOTS_GINDEX, BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX,
        BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX, BEACON_STATE_JUSTIFICATION_BITS_GINDEX,
        BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX, BEACON_STATE_SLOT_GINDEX,
        SLOTS_PER_EPOCH, SLOTS_PER_HISTORICAL_ROOT,
    },
    weigh_justification_and_finalization::{
        checkpoint::{CheckpointValue, CheckpointVariable},
        justification_bits::{JustificationBitsValue, JustificationBitsVariable},
        WeighJustificationAndFinalization,
    },
};
use ethers::types::H256;
use lighthouse_cached_tree_hash::{CacheArena, CachedTreeHash};
use lighthouse_ef_tests::{self, testing_spec};
use lighthouse_state_merkle_proof::MerkleTree;
use lighthouse_state_processing::{
    common::update_progressive_balances_cache::initialize_progressive_balances_cache,
    per_epoch_processing::altair::ParticipationCache,
};
use lighthouse_types::{
    consts::altair::TIMELY_TARGET_FLAG_INDEX, BeaconState, ChainSpec, Epoch, EthSpec, ForkName,
    Hash256, MainnetEthSpec, RelativeEpoch,
};
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        ArrayVariable, Bytes32Variable, CircuitBuilder, DefaultParameters, PlonkParameters,
        U64Variable,
    },
    utils::bytes32,
};
use snap::raw::Decoder;
use std::{fs, path::Path};

pub fn ssz_decode_state<E: EthSpec>(path: &Path, spec: &ChainSpec) -> BeaconState<E> {
    let compressed_bytes = fs::read(path).unwrap();
    let mut decoder = Decoder::new();
    let ssz_bytes = decoder.decompress_vec(&compressed_bytes).unwrap();
    BeaconState::from_ssz_bytes(ssz_bytes.as_slice(), &spec).unwrap()
}

struct Balances {
    pub total_active_balance: u64,
    pub previous_target_balance: u64,
    pub current_target_balance: u64,
}

fn extract_balances<E: EthSpec>(state: &mut BeaconState<E>, spec: &ChainSpec) -> Balances {
    // Ensure the committee caches are built.
    state
        .build_committee_cache(RelativeEpoch::Previous, spec)
        .unwrap();
    state
        .build_committee_cache(RelativeEpoch::Current, spec)
        .unwrap();
    state
        .build_committee_cache(RelativeEpoch::Next, spec)
        .unwrap();

    // Pre-compute participating indices and total balances.
    let participation_cache = ParticipationCache::new(state, spec).unwrap();
    // let sync_committee = state.current_sync_committee().unwrap().clone();
    initialize_progressive_balances_cache(state, Some(&participation_cache), spec).unwrap();

    // Justification and finalization.
    let previous_epoch = state.previous_epoch();
    let current_epoch = state.current_epoch();
    let previous_indices = participation_cache
        .get_unslashed_participating_indices(TIMELY_TARGET_FLAG_INDEX, previous_epoch)
        .unwrap();
    let current_indices = participation_cache
        .get_unslashed_participating_indices(TIMELY_TARGET_FLAG_INDEX, current_epoch)
        .unwrap();

    Balances {
        total_active_balance: participation_cache.current_epoch_total_active_balance(),
        previous_target_balance: previous_indices.total_balance().unwrap(),
        current_target_balance: current_indices.total_balance().unwrap(),
    }
}

fn compute_merkle_proof<E: EthSpec>(
    state: &mut BeaconState<E>,
    generalized_index: usize,
) -> Vec<Hash256> {
    let mut cache = state.tree_hash_cache_mut().take().unwrap();
    let leaves = cache.recalculate_tree_hash_leaves(state).unwrap();
    state.tree_hash_cache_mut().restore(cache);

    let depth = 5;
    let tree = MerkleTree::create(&leaves, depth);
    let (_, proof) = tree.generate_proof(generalized_index, depth).unwrap();

    proof
}

fn compute_block_roots_merkle_proof<E: EthSpec>(
    state: &mut BeaconState<E>,
    index: usize,
) -> Vec<Hash256> {
    let arena = &mut CacheArena::default();
    let mut cache = state.block_roots().new_tree_hash_cache(arena);
    let _ = state
        .block_roots()
        .recalculate_tree_hash_root(arena, &mut cache);

    let mut leaves = Vec::new();
    cache
        .leaves()
        .iter(arena)
        .unwrap()
        .for_each(|leaf| leaves.push(*leaf));

    let depth = 13;
    let tree = MerkleTree::create(leaves.as_slice(), depth);
    let (_, proof) = tree.generate_proof(2usize.pow(13) + index, depth).unwrap();
    proof
}

fn compute_block_roots_start_epoch_slot_to_beacon_state_proof<E: EthSpec>(
    state: &mut BeaconState<E>,
    epoch: Epoch,
) -> Vec<Hash256> {
    let block_roots_proof = compute_merkle_proof(state, BEACON_STATE_BLOCK_ROOTS_GINDEX as usize);
    let index = ((epoch.as_u64() * SLOTS_PER_EPOCH) % SLOTS_PER_HISTORICAL_ROOT) as usize;
    let block_roots_slot_proof = compute_block_roots_merkle_proof(state, index);

    [
        block_roots_slot_proof.as_slice(),
        block_roots_proof.as_slice(),
    ]
    .concat()
    .to_vec()
}

pub fn compute_beacon_state_tree_hash_root<E: EthSpec>(state: &mut BeaconState<E>) -> Hash256 {
    let mut cache = state.tree_hash_cache_mut().take().unwrap();
    let root_hash = cache.recalculate_tree_hash_root(state).unwrap();
    state.tree_hash_cache_mut().restore(cache);
    root_hash
}

pub fn get_block_root_epoch_start_slot_root<E: EthSpec>(
    state: &BeaconState<E>,
    epoch: Epoch,
) -> Hash256 {
    state.block_roots()[((epoch.as_u64() * SLOTS_PER_EPOCH) % SLOTS_PER_HISTORICAL_ROOT) as usize]
}

fn test_circuit_ssz_snappy() {
    plonky2x::utils::setup_logger();
    type L = DefaultParameters;
    const D: usize = 2;

    let spec = &testing_spec::<MainnetEthSpec>(ForkName::Capella);
    let mut state = ssz_decode_state::<MainnetEthSpec>(Path::new("./bin/pre.ssz_snappy"), spec);
    state.initialize_tree_hash_cache();
    let balances = extract_balances(&mut state, spec);

    let mut builder = CircuitBuilder::<L, D>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    let slot = state.slot().as_u64();
    let slot_proof = compute_merkle_proof(&mut state, BEACON_STATE_SLOT_GINDEX as usize);

    let beacon_state_root = compute_beacon_state_tree_hash_root(&mut state);

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.previous_justified_checkpoint().epoch.as_u64(),
        root: state.previous_justified_checkpoint().root,
    };

    let previous_justified_checkpoint_proof = compute_merkle_proof(
        &mut state,
        BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX as usize,
    );

    let current_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.current_justified_checkpoint().epoch.as_u64(),
        root: state.current_justified_checkpoint().root,
    };

    let current_justified_checkpoint_proof = compute_merkle_proof(
        &mut state,
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
        compute_merkle_proof(&mut state, BEACON_STATE_JUSTIFICATION_BITS_GINDEX as usize);

    let previous_epoch = state.previous_epoch();
    let previous_epoch_start_slot_root_in_block_roots_proof =
        compute_block_roots_start_epoch_slot_to_beacon_state_proof(&mut state, previous_epoch);

    let current_epoch = state.current_epoch();
    let current_epoch_start_slot_root_in_block_roots_proof =
        compute_block_roots_start_epoch_slot_to_beacon_state_proof(&mut state, current_epoch);

    let previous_epoch_start_slot_root_in_block_roots =
        get_block_root_epoch_start_slot_root(&state, state.previous_epoch());
    let current_epoch_start_slot_root_in_block_roots =
        get_block_root_epoch_start_slot_root(&state, state.current_epoch());

    let finalized_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: state.finalized_checkpoint().epoch.as_u64(),
        root: state.finalized_checkpoint().root,
    };

    let finalized_checkpoint_proof = compute_merkle_proof(
        &mut state,
        BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX as usize,
    );

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

    let (proof, mut output) = circuit.prove(&input);
    circuit.verify(&proof, &input, &output);

    let new_previous_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_current_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_finalized_checkpoint = output.read::<CheckpointVariable>();
    let new_justification_bits = output.read::<JustificationBitsVariable>();

    println!("outputs:");
    println!(
        "new_previous_justified_checkpoint: {:?}",
        new_previous_justified_checkpoint
    );
    println!(
        "new_current_justified_checkpoint: {:?}",
        new_current_justified_checkpoint
    );
    println!("new_finalized_checkpoint: {:?}", new_finalized_checkpoint);
    println!("new_justification_bits: {:?}", new_justification_bits);

    let post_state = ssz_decode_state::<MainnetEthSpec>(Path::new("./bin/post.ssz_snappy"), spec);

    println!("expected outputs:");
    println!(
        "new_previous_justified_checkpoint: {:?}",
        post_state.previous_justified_checkpoint()
    );
    println!(
        "new_current_justified_checkpoint: {:?}",
        post_state.current_justified_checkpoint()
    );
    println!(
        "new_finalized_checkpoint: {:?}",
        post_state.finalized_checkpoint()
    );
    println!(
        "new_justification_bits: [{}, {}, {}, {}]",
        post_state.justification_bits().get(0).unwrap(),
        post_state.justification_bits().get(1).unwrap(),
        post_state.justification_bits().get(2).unwrap(),
        post_state.justification_bits().get(3).unwrap(),
    );
}

#[allow(unused)]
fn test_circuit_sample_data() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<L, D>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    let beacon_state_root =
        bytes32!("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

    let slot = 6953401;

    let slot_proof: [H256; 5] = [
        bytes32!("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a"),
        bytes32!("96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1"),
        bytes32!("ef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f"),
        bytes32!("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217291,
        root: bytes32!("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    let previous_justified_checkpoint_proof = [
        bytes32!("0xf7b1fc5e9ef34f7455c8cc475a93eccc5cd05a3879e983a2bad46bbcbb2c71f5"),
        bytes32!("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let current_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217292,
        root: bytes32!("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1"),
    };

    let current_justified_checkpoint_proof = [
        bytes32!("0x2b913be7c761bbb483a1321ff90ad13669cbc422c8e23eccf9eb0137c8c3cf48"),
        bytes32!("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let justification_bits = JustificationBitsValue::<<L as PlonkParameters<D>>::Field> {
        bits: vec![true, true, true, true],
    };

    let justification_bits_proof = [
        bytes32!("0x1fca1f5d922549df42d4b5ca272bd4d022a77d520a201d5f24739b93f580a4e0"),
        bytes32!("0x9f1e3e59c7a4606e788c4e546a573a07c6c2e66ebd245aba2ff966b27e8c2d4f"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let previous_epoch_start_slot_root_in_block_roots_proof = [
        bytes32!("0x73dea1035b1bd431ccd1eaa893ad5f4b8488e68d2ca90615e5be0d8f7ba5a650"),
        bytes32!("0x0f7c6aa59235e573a4cdfb9411d5e4eb6255571814906c5928c016626aa2ff0a"),
        bytes32!("0xf770f73c2e01ddf6c71765e327eebb7bab0ab13f4506c736dfd6556037c0e646"),
        bytes32!("0x036f0750c86fdc21edee72b6ac1b5f728eed354c99d3b6862adf60f72bc5dcbe"),
        bytes32!("0x9730c8f3978ea7a1797603b19514e74273898f2be969ca8c583f55d14cd08d03"),
        bytes32!("0x47b601e8c14026380bdd0f716a4188e9f50a86bc58f0c342ead2a075ba8e5bc0"),
        bytes32!("0x6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        bytes32!("0x82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        bytes32!("0x30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        bytes32!("0xc9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        bytes32!("0x606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        bytes32!("0x4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        bytes32!("0xf3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        bytes32!("0xc524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        bytes32!("0xe3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        bytes32!("0x844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        bytes32!("0x2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("0x71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let current_epoch_start_slot_root_in_block_roots_proof = [
        bytes32!("c798192e5a066fe1ff3fc632bccd30a1ff47dc4d36909725db43ca6b23a5a7ba"),
        bytes32!("3161f17c79044792fc7c965a3fcb105f595bf895a44a774b871fa3017f5a36cc"),
        bytes32!("e3dddf89fa44413c3d4cf1762d7500b169116125194d96e86257cb616949560f"),
        bytes32!("3bfbdebbb29b9e066e08350d74f66116b0221c7d2c98724288a8e02bc7f937ae"),
        bytes32!("f50adbe1bff113f5d5535eee3687ac3b554af1eb56f8c966e572f8db3a61add3"),
        bytes32!("1a973e9b4fc1f60aea6d1453fe3418805a71fd6043f27a1c32a28bfcb67dd0eb"),
        bytes32!("6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        bytes32!("82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        bytes32!("30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        bytes32!("c9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        bytes32!("606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        bytes32!("4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        bytes32!("f3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        bytes32!("c524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        bytes32!("e3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        bytes32!("844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        bytes32!("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let previous_epoch_start_slot_root_in_block_roots =
        bytes32!("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1");
    let current_epoch_start_slot_root_in_block_roots =
        bytes32!("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c");

    let total_active_balance = 10;
    let previous_epoch_target_balance = 10;
    let current_epoch_target_balance = 20;

    let finalized_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217291,
        root: bytes32!("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    let finalized_checkpoint_proof = [
        bytes32!("0x26803d08d4a1a3d223ed199292fa78e62ef586391213548388375f302acfdece"),
        bytes32!("0xf0af1bff0357d4eb3b97bd6f7310a71acaff5c1c1d9dde7f20295b2002feccaf"),
        bytes32!("0x43e892858dc13eaceecec6b690cf33b7b85218aa197eb1db33de6bea3d3374c2"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    input.write::<Bytes32Variable>(beacon_state_root);
    input.write::<U64Variable>(slot);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(slot_proof.to_vec());
    input.write::<CheckpointVariable>(previous_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(previous_justified_checkpoint_proof.to_vec());
    input.write::<CheckpointVariable>(current_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(current_justified_checkpoint_proof.to_vec());
    input.write::<JustificationBitsVariable>(justification_bits);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(justification_bits_proof.to_vec());
    input.write::<U64Variable>(total_active_balance);
    input.write::<U64Variable>(previous_epoch_target_balance);
    input.write::<U64Variable>(current_epoch_target_balance);
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

    let (proof, mut output) = circuit.prove(&input);
    circuit.verify(&proof, &input, &output);

    let new_previous_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_current_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_finalized_checkpoint = output.read::<CheckpointVariable>();
    let new_justification_bits = output.read::<JustificationBitsVariable>();

    println!("outputs:");
    println!(
        "new_previous_justified_checkpoint: {:?}",
        new_previous_justified_checkpoint
    );
    println!(
        "new_current_justified_checkpoint: {:?}",
        new_current_justified_checkpoint
    );
    println!("new_finalized_checkpoint: {:?}", new_finalized_checkpoint);
    println!("new_justification_bits: {:?}", new_justification_bits);
}

fn main() {
    test_circuit_ssz_snappy();
    // test_circuit_sample_data();
}
