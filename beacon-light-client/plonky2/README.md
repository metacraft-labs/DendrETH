# The Balance Verification Task Execution and Data Flow

The Balance Verification relies heavily on a Redis instance to handle task management. This document provides an overview of the technical structure of the system, including how tasks are produced, consumed, and executed.

## Commitment Mapper

1. **Task Creation**

   - A JavaScript script feeds the commitment mapper tasks. This script downloads all the validators from the beacon chain.
   - For each validator, the script produces a proving task.
   - At every epoch, the script checks for any changed validators and creates proving tasks for them.

   The script can be executed by navigating to the `beacon-light-client/plonky2/validators_commitment_mapper_tree/get_changed_validators.ts` directory and running:

   ```
    ts-node get_changed_validators.ts --redis-host [value] --redis-port [number] --take [number] --beacon-node [value]
   ```

   Flags:

   - `--redis-host [string]`: Optional. Specifies the Redis host address. Defaults to: `127.0.0.1`
   - `--redis-port [number]`: Optional. Specifies the Redis port number. Defaults to: `6379`
   - `--take [number]`: Optional. Limits the number of validators for task creation. Useful for testing. Defaults to: `takes all`
   - `--beacon-node [array]`: Optional. Sets the beacon api url. Defaults to: `http://unstable.mainnet.beacon-api.nimbus.team`
   - `--slot [number]`: Optional. Fetches the balances for this slot. Defaults to: `undefined`
   - `--take [number]`: Optional. Sets the number of validators to take. Defaults to: `Infinity`
   - `--offset [number]`: Optional. Index offset in the validator set. Defaults to: `undefined`

2. **Task Consumption**

   - The Rust program listens for tasks in Redis and subsequently generates proofs based on each task.
   - The program can be initiated using:

   ```
   cargo run --bin commitment_mapper --release -- --redis [URI] --run_for [value] --stop-after [value] --lease-for [value]
   ```

   Flags:

   - `--redis [URI]`: Optional. Specifies the Redis connection URI. Defaults to: `redis://127.0.0.1:6379/`
   - `--stop-after [value]`: Optional. Sets how many seconds to wait until the program stops if no new tasks are found in the queue. Defaults to: `20`
   - `--lease-for [value]`: Optional. Sets for how long the task will be leased and then possibly requeued if not finished. Defaults to: `30`

3. **Cleaning Unfinished Tasks**

   - Occasionally, tasks might get leased by a worker but aren't completed. The `ligth_cleaner.ts` script in the `beacon-light-client/plonky2/validators_commitment_mapper_tree/get_changed_validators.ts` directory cleans up such tasks. Run the script using:

   ```
   ts-node light_cleaner.ts --redis-host [value] --redis-port [number]
   ```

   Flags:

   - `--redis-host [value]`: Optional. Specifies the Redis host address. Defaults to: `127.0.0.1`
   - `--redis-port [number]`: Optional. Specifies the Redis port number. Defaults to: `6379`

4. **Serialized Circuits Preparation**

   - To successfully run the `commitment_mapper` worker, serialized circuits must be present in your directory.
   - These can be added by compiling and serializing them using:

   ```
   cargo run --bin commitment_mapper_circuit_data_generation --release -- --level [value]
   ```

Flags:

- `--level [value]`: Optional. Sets the circuit level. Defaults to: `all`

## Balance Verification

1. **Circuits Serialization**
   ```
   cargo run --bin balance_verification_circuit_data_generation --release -- --level [value]
   ```

Flags:

- `--level [value]`: Optional. Sets the circuit level. Defaults to: `all`

2. **Task Creation for Getting Balances**

   - Unlike the commitment mapper that continually checks for changes, this is a one-time run script, which is executed as:

   ```
   ts-node get_balances_input.ts --redis-host [value] --redis-port [number] --take [number] --beacon-node [value]
   ```

   Flags:

   - `--redis-host [value]`: Optional. Specifies the Redis host address. Defaults to: `127.0.0.1`
   - `--redis-port [number]`: Optional. Specifies the Redis port number. Defaults to: `6379`
   - `--take [number]`: Optional. Limits the number of validators for task creation. Useful for testing. Defaults to: `takes all`
   - `--beacon-node [value]`: Optional. Sets the beacon api url. Defaults to: `http://unstable.mainnet.beacon-api.nimbus.team`

3. **Cleaning Unfinished Tasks**
   - The cleaner script resides in the same directory as above:
   ```
   ts-node light_cleaner.ts --redis-host [value] --redis-port [number]
   ```

Flags:

- `--redis-host [value]`: Optional. Specifies the Redis host address. Defaults to: `127.0.0.1`
- `--redis-port [number]`: Optional. Specifies the Redis port number. Defaults to: `6379`

4. **Task Workers for Different Levels**

   - Because these will run in the cloud, workers for each level are isolated. They can be run using:

   ```
   cargo run --bin balance_verification --release -- --redis [URI] --run_for [value] --stop-after [value] --lease-for [value] --level 0
   ```

   Each level (n) requires the n-th circuit files, and the (n-th - 1) circuit for recursive verification of the previous proof.

   Flags:

   - `--level [value]`: Required. Sets the circuit level.
   - `--redis [URI]`: Optional. Specifies the Redis connection URI. Defaults to: `redis://127.0.0.1:6379/`
   - `--run-for [value]`: Optional. Determines how long the program should run for, specified in minutes. Defaults to: `infinity`
   - `--stop-after [value]`: Optional. Sets how many seconds to wait until the program stops if no new tasks are found in the queue. Defaults to: `20`
   - `--lease-for [value]`: Optional. Sets for how long the task will be leased and then possibly requeued if not finished. Defaults to: `30`

5. **Final Proof Execution**

   - Once all levels are prepared, you can generate the final proof with the following:

   ```
   cargo run --bin final_layer --release --redis [URI]
   ```

   This is the generation of the final proof with public inputs: `withdrawalCredentials`, `stateRoot`, and `totalLockedValue`.

   Flags:

   - `--redis [URI]`: Optional. Specifies the Redis connection URI. Defaults to: `redis://127.0.0.1:6379/`
