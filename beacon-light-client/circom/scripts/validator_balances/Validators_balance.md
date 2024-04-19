The public inputs to the circuit are:

- `validatorsAccumulator`: A Merkle tree accumulator of validator public keys and Eth1 deposit indexes. Eth1 deposit index is need to validate if the validator is already part of the beacon chain.
- `stateRoot`: The current beacon state root against which the proof is made.
- `balanceSum`: The sum of all active validator balances.

The workflow of the circuit is as follows:

1. **Epoch Calculation**: The `currentEpoch` is calculated from the provided `slot`. This calculation is then constrained. A Merkle proof verifies that the `slot` aligns with the provided `stateRoot`.

2. **Index and Root Validations**: The circuit checks if the `currentEth1DepositIndex` aligns with the `stateRoot`. It also validates whether the provided `validatorsRoot` and `balancesRoot` are part of the `stateRoot`.

3. **Validator Verification**: The circuit validates all validators within `validatorsRoot`. It uses the `RangeCheck` circuit to confirm that the `currentEpoch` falls between the validator's activation and exit epochs. If a validator is inactive or their Eth1 deposit index precedes the validator's Eth1 deposit index, the bitmask for that validator is set to 0. Validators whose Eth1 deposit index is larger than the current Eth1 deposit index aren't constrained to have valid Merkle proof because they aren't yet part of the tree.

4. **Balance Index Check**: The circuit checks that provided `balancesProofIndexes` correspond to the `validatorIndex` divided by 4 plus `balancesProofIndexesRemainders`. This is because balances are stored in a 256-bit array, with each balance occupying a separate 64 bits of the 256-bit segment.

5. **Balance Validations**: It validates that all the passed balances are valid with respect to the `balancesRoot`.

6. **Validator Accumulator Validation**: The circuit verifies that the hash tree root of all passed validators and their Eth1 deposit indexes matches the passed `validatorAccumulator`.

7. **Balance Sum Calculation**: The circuit sums all the balances according to the bitmask calculated for the validators.

8. **Hashing and Output Generation**: The public values are hashed. The circuit outputs the first 253 bits of the SHA-256 hash of the public inputs to make circuit verification more cost-effective.
