# Fuzzer

## Usage

To use the fuzzer, you need to install the `cargo-fuzz` tool:

```bash
cargo install cargo-fuzz
```

## Seed Corpus

The corpus is located in the `corpus` directory. It should contain files to seed the fuzzer with example data so that it generates better inputs on the fly. To seed the corpus, for example for the compute shuffled index minimal target, run the following command:

```bash
./seed/compute_shuffled_index.sh minimal
```

This will create seed files in the corpus/compute_shuffled_index_minimal directory. To seed the corpus for the mainnet target, run the following command respectively:

```bash
./seed/compute_shuffled_index.sh mainnet
```

## Running the fuzzer with coverage (slow)

To run the compute shuffled index minimal target, run the following command:

```bash
cargo fuzz run compute_shuffled_index_minimal -- -rss_limit_mb=4096
```

The rss_limit_mb flag is optional and can be used to limit the memory usage of the fuzzer. This target needs more memory or it runs out of it so we need to increase the limit.

The mainnet version, for example, consumes even more memory so we need to increase the limit even more and moreover to adjust the default timeout:

```bash
cargo fuzz run compute_shuffled_index_mainnet -- -rss_limit_mb=8192 -timeout=100000
```

## Running the fuzzer without coverage (fast)

To run the compute shuffled index minimal target, run the following command (from the fuzz directory):

```bash
./scripts/fuzz.sh compute_shuffled_index_minimal -rss_limit_mb=4096
```

Like when using the `cargo fuzz` command, any flag can be passed to the fuzzer after the name of the fuzz target.

**Note**: The fuzzer will run much faster without coverage but it will not be able to detect any new code coverage. It is recommended to run the fuzzer with coverage from time to time to detect new code coverage according to the libfuzzer documentation.

## Differential fuzzing

Currently, the fuzz targets compare the result from the circuits against the result from the lighthouse implementation. This is how we achieve differential fuzzing and are able to detect bugs in the circuits. In order for this to work in a sensible amount of time, the coverage is disabled. This way each test passes for the amount of time it takes to run in the test engine.
