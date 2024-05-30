# `circuits` Crate Overview

The `circuits` crate houses all circuits pertinent to the application. Let's delve into the specifics of each:

---

## Commitment Mapper

### First Level

**Location:** `src/validators_commitment_mapper/first_level.rs`

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

**Location:** `circuits/src/withdrawal_credentials_balance_aggregator/inner_level`

**Description:** Defines the inner level of the balance verification circuit.

**Workflow:**

1. **Circuit Definition:** The circuit uses `BasicRecursiveInnerCircuitTarget` defined in `circuits/src/common_targets.rs` which basically defines two proofs as targets.
2. **Proof Verification:** Both proofs undergo verification.
3. **Hash Computation:** Hash values from the proofs are extracted and combined to produce a new level hash.
4. **Balance Summation:** Sums from proofs are aggregated.
5. **Credential & Epoch Validation:** The `withdrawal_credentials` from the proofs are asserted for equality, similar to the `current_epoch`.

---

## Withdrawal credentials balance aggregator final layer

**Location:** `circuits/src/withdrawal_credentials_balance_aggregator/final_layer.rs`

**Description:** This circuit consolidates the results from the final levels of both the balance verification and commitment mapper circuits.

**Workflow:**

1. **Proof Verification:** The circuit first verifies the provided proofs and extracts the public_inputs from both.

2. **Hash Assertion:** The circuit then asserts equality between the hashes of `commitment_mapper_poseidon_root` and `balances_validator_poseidon_root`.

3. **Final Tree Hash Computation:** The SHA-256 hashes, in conjunction with the `validator_size`, are used to compute the final tree hash of the validators and balances trees.

4. **Merkle Proof Verification:** The final sha256 hashes of the validators and balances tree undergo Merkle proof verification against the given state root.

5. **Slot Verification:** The circuit invokes the `assert_slot_is_in_epoch` function. This ensures the provided slot is within the `current_epoch` derived from the balance verification proof.

6. **Slot to Bits Conversion:** The slot value is converted to bits, which then undergoes validation against the state root.

7. **Public Inputs Registration:** Lastly, the `state_root`, `withdrawal_credentials`, `balance_sum`, `number_of_non_activated_validators`, `number_of_active_validators_bits`, `number_of_exited_validators_bits`, `number_of_slashed_validators_bits` are registered as public inputs.

---

## Deposits Accumulator Balance Aggregator

The purpose of this circuit is to compute the total value locked (TVL) of all validators that have deposited through the `ValidatorsAccumulator` contract on-chain. This computation is done within the `deposits_accumulator_balance_aggregator` circuit, which has both first-level and inner-level components.

Read more detailed docs here: [Deposit accumulator balance aggregator description](./src/deposits_accumulator_balance_aggregator/README.md)
