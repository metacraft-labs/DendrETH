FROM diva-base

RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c 'cd beacon-light-client/plonky2/crates/circuit_executables && cargo run --bin commitment_mapper_circuit_data_generation --release'
RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c 'cd beacon-light-client/plonky2/crates/circuit_executables && cargo build --bin commitment_mapper --release'

COPY --chmod=0755 run.sh /
CMD ["/run.sh"]
