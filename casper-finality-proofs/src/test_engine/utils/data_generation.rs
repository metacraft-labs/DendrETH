use lighthouse_cached_tree_hash::{CacheArena, CachedTreeHash};
use lighthouse_state_merkle_proof::MerkleTree;
use lighthouse_state_processing::{
    common::update_progressive_balances_cache::initialize_progressive_balances_cache,
    per_epoch_processing::altair::ParticipationCache,
};
use lighthouse_types::{
    consts::altair::TIMELY_TARGET_FLAG_INDEX, BeaconState, ChainSpec, Epoch, Eth1Data, EthSpec,
    Hash256, MainnetEthSpec, RelativeEpoch,
};

use crate::constants::{
    BEACON_STATE_BLOCK_ROOTS_GINDEX, SLOTS_PER_EPOCH, SLOTS_PER_HISTORICAL_ROOT,
};

pub struct Balances {
    pub total_active_balance: u64,
    pub previous_target_balance: u64,
    pub current_target_balance: u64,
}

pub fn extract_balances<E: EthSpec>(state: &mut BeaconState<E>, spec: &ChainSpec) -> Balances {
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

pub fn compute_merkle_proof<E: EthSpec>(
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

pub fn compute_block_roots_start_epoch_slot_to_beacon_state_proof<E: EthSpec>(
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

pub fn init_beacon_state(eth1_data: Eth1Data, spec: &ChainSpec) -> BeaconState<MainnetEthSpec> {
    BeaconState::new(0, eth1_data, spec)
}
