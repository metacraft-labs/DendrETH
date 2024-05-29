# `circuits` Crate Overview

The `circuits` crate houses all circuits pertinent to the application. Let's delve into the specifics of each:

---

## Commitment Mapper

### First Level

**Location:** `src/validators_commitment_mapper/first_level`

**Description:** A general-purpose commitment mapping circuit establishes a correspondence between currentValidatorsRoot value (a SSZ hashTreeRoot) and a Poseidon root hash of an equivalent tree of records (currentValidatorsRootPoseidon)

**Workflow:**

1. **Target Definitions:** Defines the `ValidatorsCommitmentMapperTarget` and marks the inputs and outputs of the circuit
2. **Circuit Definition:** The `Circuit` trait is implemented for `ValidatorsCommitmentMapperFirstLevel` and the define function merklelize_validator_target for hashing it with sha256 and also hashes it with poseidon hash. Returning the two outputs `sha256_hash_tree_root` and `poseidon_hash_tree_root`.
3. **Hashing Procedures:** The circuit uses `hash_validator_sha256_or_zeroes` and `hash_validator_poseidon_or_zeroes` defined in `circuits/src/utils/circuit/hashing/merkle/poseidon.rs` and `circuits/src/utils/circuit/hashing/merkle/sha256.rs` respectively.
4. **Target Mapping:** To produce the sha256 hash tree root the circuit uses `merklelize_validator_target` defined in `circuits/src/utils/circuit/hashing/merkle/sha256.rs`

#### Inner Level

**Location:** `src/validators_commitment_mapper/inner_level`

**Description:** This layer builds the commitment mapper's inner circuit, handling the merger of proofs and computing the subsequent level of the tree.

**Workflow:**

1. **Circuit Definition:** The circuit uses `BasicRecursiveInnerCircuitTarget` defined in `circuits/src/common_targets.rs` which basically defines two proofs as targets.
2. **Proof Processing:** The circuit verifies the authenticity of the two submitted proofs.
3. **Hash Tree Root Extraction:** The circuit extracts the hashes from the proofs and combines them, laying the foundation for the next tree level.
4. **Public Inputs:** Once the combined hash tree root is determined, it's registered as the public inputs of the new proof.

---

## Withdrawal credentials balance aggregator

**Description:** Computes the total value locked of the validators with given withdrawal credentials

### First Level

**Location:** `circuits/src/withdrawal_credentials_balance_aggregator/first_level`

**Workflow:**

1. **Target Definitions:** Defines the `ValidatorBalanceVerificationTargets`
2. **Circuit Inputs:** Circuit accepts various inputs such as validators, balances_leaves, withdraw_credentials, current_epoch, and non_zero_validator_leaves_mask.
3. **Hashing Process:** The `hash_tree_root_sha256` circuit calculates the hash tree root of the balance leaves.
4. **Validator Hashing:** The `hash_validator_poseidon_or_zeroes` circuit computes the root for each validator.
5. **Balance Summation:** The circuit processes the sum of balances, factoring in conditions based on validator `withdrawal_credentials` and activation status.

### Inner Level

**Location:** `build_balance_inner_level_circuit.rs`

**Description:** Defines and builds the inner level of the balance verification circuit.

**Workflow:**

1. **Circuit Definition:** The circuit uses `BasicRecursiveInnerCircuitTarget` defined in `circuits/src/common_targets.rs` which basically defines two proofs as targets.
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

## Deposits Accumulator Balance Aggregator

The purpose of this circuit is to compute the total value locked (TVL) of all validators that have deposited through the `ValidatorsAccumulator` contract on-chain. This computation is done within the `deposits_accumulator_balance_aggregator` circuit, which has both first-level and inner-level components.

### First Level

**Description:**

The circuit takes the following inputs:

- `pub validator`: `ValidatorTarget`
- `pub commitment_mapper_root`: `HashOutTarget`
- `pub commitment_mapper_proof`: `PoseidonMerkleBranchTarget<40>`
- `pub validator_gindex`: `BigUintTarget` (serialized/deserialized with `biguint_to_str` and `parse_biguint`)
- `pub eth1_deposit_index`: `BigUintTarget` (serialized/deserialized with `biguint_to_str` and `parse_biguint`)
- `pub genesis_fork_version`: `[BoolTarget; 32]` (serialized/deserialized with `serde_bool_array_to_hex_string`)
- `pub validator_deposit`: `DepositTargets`
- `pub deposit_commitment_mapper_root`: `HashOutTarget`
- `pub validator_deposit_proof`: `PoseidonMerkleBranchTarget<32>`
- `pub validator_deposit_gindex`: `BigUintTarget` (serialized/deserialized with `biguint_to_str` and `parse_biguint`)
- `pub balance_tree_root`: `Sha256Target` (serialized/deserialized with `serde_bool_array_to_hex_string`)
- `pub balance_leaf`: `Sha256Target` (serialized/deserialized with `serde_bool_array_to_hex_string`)
- `pub balance_proof`: `Sha256MerkleBranchTarget<22>` (serialized/deserialized with `serde_bool_array_to_hex_string_nested`)
- `pub is_dummy`: `BoolTarget`
- `pub current_epoch`: `BigUintTarget` (serialized/deserialized with `biguint_to_str` and `parse_biguint`)
- `pub bls_signature_proof`: `ProofWithPublicInputsTarget<2>`

**Process:**

1. **Merkle Proof Validation:**

   - If the deposit is real (needed to represent zero deposits to pad the tree), validate the Merkle proof using Poseidon for the `validator_deposit`.

2. **BLS Signature Validation:**

   - Compute the domain using the `genesis_fork_version`.
   - Compute the message the validator signed using the `validator_deposit.deposit_message_root` and the domain.
   - Verify the `bls_signature_proof` and assert its public inputs against the `validator_deposit.pubkey`, `validator_deposit.signature`, and the computed message.

3. **Deposit Processing:**

   - Consider the deposit processed if `validator_deposit.deposit_index` <= `eth1_deposit_index`.
   - If the deposit is processed and the signature is valid, confirm that the validator is definitely on-chain.

4. **Commitment Mapper Validation:**

   - Check the Merkle proof against the `commitment_mapper`.
   - Ensure that the pubkeys of the deposit and the validator from the commitment mapper are the same.

5. **Balance Validation:**

   - Validate the `balance_leaf` against the `balances_root` using the formula: validator_index / 4 and take the balance in the leaf at `validator_index % 4`.

6. **Updating Counts:**
   - Update the `active_count`, `exited_count`, `slashed_count`.

### Handling Dummy Validators

- **Dummy Validator Handling:**
  - The `is_dummy` flag is used to identify dummy validators.
  - If `is_dummy` is set, the validator is asserted to have the `max_pubkey`.
  - Dummy validators are not included in the balance sum calculation.

### Returned Node Object

The circuit returns a `node` object which contains the following:

- **Leftmost Range Object:**

  - `pubkey`: The public key of the validator.
  - `deposit_index`: The index of the validator's deposit.
  - `is_counted`: A boolean indicating if the Merkle proof should be checked.
  - `is_dummy`: A boolean indicating if this is a dummy validator.

- **Rightmost Range Object:**

  - Similar to the leftmost range object with the same attributes.

- **Accumulated Data:**

  - `balance_sum`: The sum of balances of all processed deposits, excluding dummy validators.
  - `deposits_count`: The count of deposits processed.
  - `validator_stats`:
    - `non_activated_validators_count`: Count of non-activated validators.
    - `active_validators_count`: Count of active validators.
    - `exited_validators_count`: Count of exited validators.
    - `slashed_validators_count`: Count of slashed validators.

- **Node Metadata:**
  - `current_epoch`: The current epoch when the node was processed.
  - `eth1_deposit_index`: The deposit index on the Ethereum 1.0 chain.
  - `commitment_mapper_root`: The root hash of the commitment mapper.
  - `balances_root`: The root hash of the balance tree.
  - `deposits_mapper_root`: The root hash of the deposits commitment mapper.
  - `genesis_fork_version`: The genesis fork version.

Sure! Here's a detailed document for the inner level of the `DepositsAccumulatorBalanceAggregator` circuit based on the provided code:

---

## Deposits Accumulator Balance Aggregator - Inner Level

The inner level of the `DepositsAccumulatorBalanceAggregator` circuit combines and verifies the outputs from two first-level circuits, ensuring the aggregated results are correctly computed and ordered.

### Description

This circuit aggregates the results from two first-level circuits, ensuring that the validators' public keys and deposit indices are correctly ordered and that the accumulated data is properly computed without double-counting.

### Inputs

The circuit takes the following inputs:

- **Proofs:**

  - `proof1`: The first proof generated from the first-level circuit.
  - `proof2`: The second proof generated from the first-level circuit.

- **Verifier Circuit Target:**
  - `constants_sigmas_cap`: The constant Merkle cap for the verifier.
  - `circuit_digest`: The circuit digest for the verifier.

### Process

1. **Verification of Proofs:**

   - Verify both `proof1` and `proof2` using the verifier circuit target:
     ```rust
     builder.verify_proof::<Self::C>(&proof1, &verifier_circuit_target, &circuit_data.common);
     builder.verify_proof::<Self::C>(&proof2, &verifier_circuit_target, &circuit_data.common);
     ```

2. **Extract Node Data:**

   - Extract `NodeTargets` from the public inputs of both proofs:
     ```rust
     let left_node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(&proof1.public_inputs).node;
     let right_node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(&proof2.public_inputs).node;
     ```

3. **Data Consistency Checks:**

   - Ensure that the data is passed through consistently between nodes:
     ```rust
     connect_pass_through_data(builder, &left_node, &right_node);
     ```

4. **Order Validation:**

   - Ensure that the public keys are in monotonic order and deposit indices are strictly monotonic:
     ```rust
     let pk_are_monotonic_ordering = cmp_pubkey(builder, left_node.rightmost.pubkey, right_node.leftmost.pubkey);
     builder.connect(pk_are_monotonic_ordering.target, _true.target);
     ```

5. **Dummy Proof Handling:**

   - Check if the right node is a dummy proof:
     ```rust
     let right_is_zero_proof = is_dummy_proof(builder, &right_node);
     ```

6. **Accumulate Data:**
   - Accumulate the data from both nodes while avoiding double-counting:
     ```rust
     let accumulated_pre_discount = accumulate_data(builder, &left_node, &right_node);
     let accumulated_final = account_for_double_counting(builder, accumulated_pre_discount, &left_node, &right_node);
     ```

### Returned Node Object

The circuit returns a `NodeTargets` object which contains the following:

- **Leftmost and Rightmost Range Objects:**

  - `leftmost`: Contains the leftmost public key, deposit index, and dummy status.
  - `rightmost`: Contains the rightmost public key, deposit index, and dummy status.
  - Both range objects inherit bounds data from their respective child nodes.

- **Accumulated Data:**

  - `balance_sum`: The sum of balances from all processed deposits.
  - `deposits_count`: The count of deposits processed.
  - `validator_stats`:
    - `non_activated_validators_count`: Count of non-activated validators.
    - `active_validators_count`: Count of active validators.
    - `exited_validators_count`: Count of exited validators.
    - `slashed_validators_count`: Count of slashed validators.

- **Node Metadata:**
  - `current_epoch`: The current epoch when the node was processed.
  - `eth1_deposit_index`: The deposit index on the Ethereum 1.0 chain.
  - `commitment_mapper_root`: The root hash of the commitment mapper.
  - `deposits_mapper_root`: The root hash of the deposits commitment mapper.
  - `balances_root`: The root hash of the balance tree.
  - `genesis_fork_version`: The genesis fork version.

### Helper Functions

- **is_dummy_proof:** Checks if a given node is a dummy proof.
- **connect_pass_through_data:** Connects pass-through data between left and right nodes.
- **cmp_pubkey:** Compares two public keys to ensure monotonic order.
- **are_pubkeys_equal:** Checks if two public keys are equal.
- **inherit_bounds_data_from_children:** Inherits bounds data from child nodes.
- **calc_counted_data:** Calculates counted data for validators.
- **has_same_pubkey_and_is_counted:** Checks if a given public key is counted.
- **accumulate_data:** Accumulates data from left and right nodes.
- **accumulate_validator_stats:** Accumulates validator statistics.
- **account_for_double_counting:** Accounts for double-counting in accumulated data.

## Deposits Accumulator Balance Aggregator - Final Layer

The final layer of the `DepositsAccumulatorBalanceAggregator` circuit consolidates the proofs and accumulated data from previous layers and verifies the integrity and correctness of the overall aggregation process.

### Description

This circuit integrates the results from the inner-level circuits and verifies the consistency of the aggregated data with the global state and block information. It ensures that the total value locked (TVL) and validator statistics are correctly accumulated and linked to the on-chain state.

### Inputs

The circuit takes the following inputs:

- **Proofs:**

  - `deposit_accumulator_root_proof`: Proof from the inner-level `DepositAccumulatorBalanceAggregator`.
  - `commitment_mapper_root_proof`: Proof from the `ValidatorsCommitmentMapper`.
  - `deposit_commitment_mapper_root_proof`: Proof from the `DepositsCommitmentMapper`.

- **State Roots and Branches:**

  - `block_root`: `Sha256Target` - The block root hash.
  - `state_root`: `Sha256Target` - The state root hash.
  - `state_root_branch`: `Sha256MerkleBranchTarget<3>` - Merkle proof for the state root.
  - `validators_branch`: `Sha256MerkleBranchTarget<5>` - Merkle proof for the validators.
  - `balance_branch`: `Sha256MerkleBranchTarget<5>` - Merkle proof for the balance.
  - `execution_block_number_branch`: `Sha256MerkleBranchTarget<5>` - Merkle proof for the execution block number.
  - `slot_branch`: `Sha256MerkleBranchTarget<5>` - Merkle proof for the slot.
  - `eth1_deposit_index_branch`: `Sha256MerkleBranchTarget<5>` - Merkle proof for the ETH1 deposit index.

- **BigInt Inputs:**
  - `execution_block_number`: `BigUintTarget` - The block number of the execution.
  - `slot`: `BigUintTarget` - The slot number.

### Process

1. **Verification of Proofs:**

   - Verify the proofs from the inner-level circuits and the commitment mappers:
     ```rust
     builder.verify_proof::<Self::C>(&deposit_accumulator_root_proof, &verifier_deposit_accumulator_inner, &deposit_accumulator_inner_circuit_data.common);
     builder.verify_proof::<Self::C>(&commitment_mapper_root_proof, &verifier_commitment_mapper, &commitment_mapper_circuit_data.common);
     builder.verify_proof::<Self::C>(&deposit_commitment_mapper_root_proof, &verifier_deposit_commitment_mapper, &deposit_commitment_mapper_circuit_data.common);
     ```

2. **Extract Node Data:**

   - Extract `NodeTargets` from the public inputs of the `deposit_accumulator_root_proof`:
     ```rust
     let node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(&deposit_accumulator_root_proof.public_inputs).node;
     ```

3. **Data Consistency Checks:**

   - Ensure consistency between the proofs and the extracted node data:
     ```rust
     builder.connect_hashes(commitment_mapper_root_public_inputs.poseidon_hash_tree_root, node.commitment_mapper_root);
     builder.connect_hashes(deposit_commitment_mapper_root_public_inputs.poseidon_hash_tree_root, node.deposits_mapper_root);
     ```

4. **Merkle Proof Validations:**

   - Validate the state root, block root, and other branches against their respective Merkle proofs:
     ```rust
     assert_merkle_proof_is_valid_const_sha256(builder, &input.state_root, &input.block_root, &input.state_root_branch, 11);
     assert_merkle_proof_is_valid_const_sha256(builder, &commitment_mapper_root_public_inputs.sha256_hash_tree_root, &input.state_root, &input.validators_branch, 86);
     assert_merkle_proof_is_valid_const_sha256(builder, &node.balances_root, &input.state_root, &input.balance_branch, 44);
     ```

5. **Slot and Range Verification:**

   - Verify the slot number and ensure it is within the valid range:
     ```rust
     verify_slot_is_in_range::<Self::F, Self::C, { Self::D }>(builder, &input.slot, &node.current_epoch);
     ```

6. **Public Inputs Hash Calculation:**

   - Calculate the public inputs hash and register it:

     ```rust
     let public_inputs_hash = sha256(builder, &[
         input.block_root.as_slice(),
         block_number_bits.as_slice(),
         deposit_commitment_mapper_root_public_inputs.sha256_hash_tree_root.as_slice(),
         deposit_count_bits.as_slice(),
         final_sum_bits.as_slice(),
         number_of_non_activated_validators_bits.as_slice(),
         number_of_active_validators_bits.as_slice(),
         number_of_exited_validators_bits.as_slice(),
         number_of_slashed_validators_bits.as_slice(),
     ].concat());

     // Mask the last 3 bits in big endian as zero
     public_inputs_hash[0] = builder._false();
     public_inputs_hash[1] = builder._false();
     public_inputs_hash[2] = builder._false();

     let public_inputs_hash_bytes = public_inputs_hash.chunks(8).map(|x| builder.le_sum(x.iter().rev())).collect_vec();

     builder.register_public_inputs(&public_inputs_hash_bytes);
     ```

### Returned Node Object

The circuit returns a `DepositAccumulatorBalanceAggregatorFinalLayerTargets` object which contains the following:

- **Proofs:**

  - `deposit_accumulator_root_proof`: Proof from the inner-level `DepositAccumulatorBalanceAggregator`.
  - `commitment_mapper_root_proof`: Proof from the `ValidatorsCommitmentMapper`.
  - `deposit_commitment_mapper_root_proof`: Proof from the `DepositsCommitmentMapper`.

- **State Roots and Branches:**

  - `block_root`: The block root hash.
  - `state_root`: The state root hash.
  - `state_root_branch`: Merkle proof for the state root.
  - `validators_branch`: Merkle proof for the validators.
  - `balance_branch`: Merkle proof for the balance.
  - `execution_block_number_branch`: Merkle proof for the execution block number.
  - `slot_branch`: Merkle proof for the slot.
  - `eth1_deposit_index_branch`: Merkle proof for the ETH1 deposit index.

- **BigInt Inputs:**
  - `execution_block_number`: The block number of the execution.
  - `slot`: The slot number.

### Helper Functions

- **assert_merkle_proof_is_valid_const_sha256:** Asserts the validity of a constant SHA-256 Merkle proof.
- **verify_slot_is_in_range:** Verifies if the slot is within a valid range.
- **sha256:** Computes the SHA-256 hash.
- **target_to_le_bits:** Converts a target to little-endian bits.
- **biguint_to_bits_target:** Converts a BigUint target to bits.
