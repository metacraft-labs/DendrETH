# Test Engine

The test engine is a tool for running unit tests for plonky2 circuits.

## Usage

- ### Types

  Contains struct `TestData` which is used to serialize input and output data for unit tests. It is used in the circuit's wrapper. Each `TestData` is stored in `src/test_engine/types/`.

  Example:

  ```rust
  pub struct Inputs {
      pub a: U256,
      pub b: U256,
  }

  pub struct Outputs {
      pub c: U256,
  }

  pub struct TestData {
      pub inputs: Inputs,
      pub outputs: Outputs,
  }
  ```

- ### JSON

  Contains input and output data for unit tests based on the circuit `TestData` struct.

  Example:

  ```json
  {
    "inputs": {
      "a": "0xa",
      "b": "0x14"
    },
    "outputs": {
      "c": "0x1e"
    }
  }
  ```

  To add a test which is expected to fail on a circuit level, name the JSON file with a trailing `_fail.json`. The test engine will expect the circuit to fail and will mark the test as successful if it does.

- ### Wrappers

  To test a circuit, create a wrapper in `src/test_engine/wrappers/`. It represents a function that writes input data to the circuit and asserts its outputs. It uses `TestData` data to assert that the circuit is working correctly.

  `path` is an argument to the `wrapper()` method, received from the test engine. It is the path to the JSON file containing the input and output data for the test.

  To use the serialized data:

  ```rust
  let json_data: TestData = read_fixture::<TestData>(path);

  input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
      json_data.inputs.a.as_u64(),
  ));

  input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
      json_data.inputs.b.as_u64(),
  ));
  ```

  Then use `assert_equal!` to validate that the output of the circuit corresponds to the expected output:

  ```rust
  let sum = output.read::<Variable>();
  assert_equal!(
      sum,
      <L as PlonkParameters<D>>::Field::from_canonical_u64(json_data.outputs.c.as_u64())
  );
  ```

- ### Setup

  When a new circuit is created with the corresponding setup mentioned above, the data should be registered in the test engine. To do this, add the following to `src/test_engine/utils/setup.rs`:

  1. Add the circuit to the `TestWrappers` enum:
     ```rust
     pub enum TestWrappers {
         NewTestCircuit,
         ...
     }
     ```
  2. Register its wrapper `wrapper_new_test()` method to the `map_test_to_wrapper`:
     ```rust
     pub fn map_test_to_wrapper(test: TestWrappers) -> ... {
         match test {
             TestWrappers::NewTestCircuit => Box::new(|path| wrapper_new_test(path.as_str())),
             ...
         }
     }
     ```
  3. Register the circuit and its tests path to the `init_tests()`:
     ```rust
     pub fn init_tests() -> Vec<TestCase> {
         ...
         tests.push(TestCase::new(
             TestWrappers::NewTestCircuit,
             "./src/test_engine/tests/new_test/".to_string(),
         ));
         ...
     }
     ```

- ### Running tests

  To run all tests, use:

  ```sh
  cargo run --bin test_engine --release
  ```

  To run a specific test, use:

  ```sh
  cargo run --bin test_engine --release -- NewTestCircuit
  ```

  where `NewTestCircuit` is the name of the registered circuit in `TestWrappers` enum in `src/test_engine/utils/setup.rs`.

  If a test fails, the console will print its name in red and after all tests have finished it will print out the circuit's name, the json file and the error.

    <style>
        rb { background-color: red; font-weight: bold }
        r { color: red; font-weight: bold }
        b { color: lightblue; font-weight: bold }
        g { color: lightgreen }
        y { color: yellow }
    </style>

  > Example:
  >
  > Running circuit: <b>WrapperTest</b>\
  > -> <g>sum_100.json</g>\
  > -> <rb>sum_30.json</rb>
  >
  > <r>Failed tests:</r>\
  > -> <b>[WrapperTest]</b> <y>sum_30.json</y>: Error: 30 != 2590\
  > &nbsp;\- at src/test_engine/wrappers/>wrapper_test.rs:31:5
