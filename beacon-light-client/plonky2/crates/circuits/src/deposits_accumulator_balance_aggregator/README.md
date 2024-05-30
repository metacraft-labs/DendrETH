# Work in progress

# Deposits Accumulator Balance Aggregation Proof

The objective of these circuits is to provide a proof for the aggregation of balance and status data for validators that have at some point deposited in our contracts and reside within the contract's deposits accumulator. The computation is done recursively and the work is distributed among 3 circuits for efficiency.

The first level circuit is responsible for processing a single deposit (leaf) of the accumulator tree. It also outputs some of its inputs back for later verification and to ensure all sibling proofs are based on the same data.

The inner level is concerned with concatenating two equally sized deposit ranges, accumulating the relevant data from the lower level and accounting for potential double counting within the computation.

Finally, the final layer circuit's job is to prove the computation was in fact done for a certain block by performing merkle proofs against its hash tree root. It also hashes its public inputs using sha256 for easier verification on chain.

## First Level Circuit

### Algorithm
- Verifies a proof that verifies the BLS signature of the deposit.
- Asserts that the BLS verification proof was made for the correct message and pubkey.
- Computes the packed poseidon hash of the deposit and performs a merkle proof against the mapped poseidon deposists accumulator hash tree root in our deposits accumulator commitment mapper.
- Computes the poseidon hash tree root of the validator and performs a merkle proof against the mapped poseidon validators hash tree root in our validators commitment mapper, but only if the deposit is real and the validator is on chain (this corresponds to the condition `deposit_index <= eth1_data.deposit_index and bls_signature_is_valid`).
- Computes the generalized index of the balances leaf of the validator (`validators_gindex / 4`) and performs a merkle proof against the balances hash tree root, but only if the deposit is real and the validator is on chain.
- Extracts the validator's balance from the balances ssz leaf.
- Computes validator status bits (is_non_activated, is_active, is_exited, is_slashed).
- Outputs if the deposit is real, zeroes out everything otherwise:
	- **validator balance** - equals the extracted balance if the pubkey is owned by an active validator
	- **validator status bits** - zeroes if it's not a validator
	- **is_counted (whether the validator's stats are already accumulated)** - validator is on chain
	- **accumulated_data (balance, deposits_count, validator_status_stats)** - (validator balance, 1, validator status bits)

### Private Inputs
- **validator** - The deposit's public key's corresponding validator
- **commitment_mapper_proof**
- **validator_gindex** - The generalized index of the deposit's corresponding validator
- **deposit** - The deposit data, as is within the accumulator
- **validator_deposit_proof**
- **validator_deposit_gindex**
- **balance_leaf** - the balances leaf the validator's balance is contained in
- **balance_proof**
- **is_dummy** - The proof is used for padding the binary tree
- **current_epoch** - The block's epoch
- **eth1_deposit_index**
- **commitment_mapper_root**
- **deposits_commitment_mapper_root**
- **balances_root**
- **genesis_fork_version**
- **bls_verification_proof**

### Public Inputs
- **current_epoch (pass-through)**
- **eth1_deposit_index (pass-through)**
- **commitment_mapper_root (pass-through)**
- **deposits_commitment_mapper_root (pass-through)**
- **balances_root (pass-through)**
- **genesis_fork_version (pass-through)**
- **leftmost_deposit**
- **rightmost_deposit**
- **accumulated_data**

## Inner Level Circuit

## Final Layer Circuit


