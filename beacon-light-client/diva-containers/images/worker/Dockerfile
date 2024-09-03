FROM diva-base

RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c 'cd beacon-light-client/plonky2/crates && cargo build --release'
RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c 'cd beacon-light-client/plonky2/crates/circuit_executables && cargo run --bin deposit_accumulator_balance_aggregator_diva_circuit_data_generation --release'

COPY --chmod=0755 run.sh /
CMD ["/run.sh"]
