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

T
This will create seed files in the corpus/compute_shuffled_index_minimal directory. To seed the corpus for the mainnet target, run the following command respectively:

```bash
./seed/compute_shuffled_index.sh mainnet
```

## Running the fuzzer

To run the compute shuffled index minimal target, run the following command:

```bash
cargo fuzz run compute_shuffled_index_minimal -- -rss_limit_mb=4096
```

The rss_limit_mb flag is optional and can be used to limit the memory usage of the fuzzer. This target needs more memory or it runs out of it so we need to increase the limit.

The mainnet version, for example, consumes even more memory so we need to increase the limit even more:

```bash
cargo fuzz run compute_shuffled_index_mainnet -- -rss_limit_mb=8192
```
