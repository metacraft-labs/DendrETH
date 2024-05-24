# `circuits` Crate Overview

The `circuits` crate houses all circuits pertinent to the application. Let's delve into the specifics of each:

---

## Commitment Mapper

**Location:** `validator_commitment_mapper.rs`

**Description:** A general-purpose commitment mapping circuit establishes a correspondence between currentValidatorsRoot value (a SSZ hashTreeRoot) and a Poseidon root hash of an equivalent tree of records (currentValidatorsRootPoseidon)

**Workflow:**

1. **Target Definitions:** Defines the `ValidatorCommitmentTargets` and ensures compliance with `ReadTargets` and `WriteTargets` traits for serialization purposes.
2. **Circuit Initialization:** The `validator_commitment_mapper` function initializes the circuit, converting `validator_sha256` inputs to `poseidon_inputs`.
3. **Hashing Procedures:** The circuit uses both `hash_tree_root_validator_sha256` and `hash_tree_root_validator_poseidon` circuits.
4. **Target Mapping:** The circuit produces `validator_poseidon_mapped` using the `bits_to_biguint_target` and `revers_endianness()`.
5. **Hash Tree Roots:** The `hash_tree_root_validator_poseidon` circuit's targets are connected, producing the hash_tree_roots as outputs.
6. **Building and public inputs registration:** Located in `build_commitment_mapper_first_level_circuit.rs` registers the `poseidon_hash_tree_root` and `sha256_hash_tree_root` as public inputs and defines trait for accessing them from proof.

---

## Commitment Mapper (Inner Level)

**Location:** `build_commitment_mapper_inner_level_circuit.rs`

**Description:** This layer builds the commitment mapper's inner circuit, handling the merger of proofs and computing the subsequent level of the tree.

**Workflow:**

1. **Circuit Definition:** Targets within this circuit are designed to handle two proofs and produce two hash_tree_roots.
2. **Proof Processing:** The circuit verifies the authenticity of the two submitted proofs.
3. **Hash Tree Root Extraction:** The circuit extracts the hashes from the proofs and combines them, laying the foundation for the next tree level.
4. **Public Inputs:** Once the combined hash tree root is determined, it's registered as the public inputs of the new proof.

---

## Balance Verification Circuit

**Location:** `validator_balance_circuit.rs`

**Description:** Computes the total value locked and the balances tree.

**Workflow:**

1. **Target Definitions:** Defines the `ValidatorBalanceVerificationTargets` and ensures compliance with serialization traits.
2. **Circuit Inputs:** Circuit accepts various inputs such as validator data in the poseidon format, the balances leaves, withdraw_credentials, current_epoch, and validator_is_zero flags.
3. **Hashing Process:** The `hash_tree_rooot` circuit calculates the hash tree root of the balance leaves.
4. **Validator Hashing:** The `hash_tree_root_validator_poseidon` circuit computes the root for each validator.
5. **Balance Summation:** The circuit processes the sum of balances, factoring in conditions based on validator `withdrawal_credentials` and activation status.
6. **Building and public inputs registration:** Located in `build_validator_balance_circuit.rs` registers the `range_total_value`, `range_balances_root`, `withdrawal_credentials`, `range_validator_commitment`, `current_epoch` as public inputs and defines trait for accessing them from proof.

---

## Balance Verification (Inner Level)

**Location:** `build_balance_inner_level_circuit.rs`

**Description:** Defines and builds the inner level of the balance verification circuit.

**Workflow:**

1. **Circuit Definition:** The circuit's targets encompass `proof1`, `proof2`, and `verifier_circuit_targets`.
2. **Proof Verification:** Both proofs undergo verification.
3. **Hash Computation:** Hash values from the proofs are extracted and combined to produce a new level hash.
4. **Balance Summation:** Sums from proofs are aggregated.
5. **Credential & Epoch Validation:** The `withdrawal_credentials` from the proofs are asserted for equality, similar to the `current_epoch`.

---

## Final Circuit

**Location:** `build_final_circuit.rs`

**Description:** This circuit consolidates the results from the final levels of both the balance verification and commitment mapper circuits.

**Workflow:**

1. **Proof Verification:** The circuit first verifies the provided proofs and extracts the public_inputs from both.

2. **Hash Assertion:** The circuit then asserts equality between the hashes of `commitment_mapper_poseidon_root` and `balances_validator_poseidon_root`.

3. **Final Tree Hash Computation:** The SHA-256 hashes, in conjunction with the `validator_size`, are used to compute the final tree hash of the validators and balances trees.

4. **Merkle Proof Verification:** The final sha256 hashes of the validators and balances tree undergo Merkle proof verification against the given state root.

5. **Slot Verification:** The circuit invokes the `verify_slot_is_in_range` function. This ensures the provided slot is within the `current_epoch` derived from the balance verification proof.

6. **Slot to Bits Conversion:** The slot value is converted to bits, which then undergoes validation against the state root.

7. **Public Inputs Registration:** Lastly, the `state_root`, `withdrawal_credentials`, and `balance_sum` are registered as public inputs.

---
