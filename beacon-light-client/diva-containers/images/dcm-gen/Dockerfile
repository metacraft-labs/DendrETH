FROM diva-base

RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' --command sh -c 'cd /app/beacon-light-client/plonky2/crates/circuit_executables && cargo build --bin pubkey_commitment_mapper --release'

COPY --chmod=0755 run.sh /
CMD ["/run.sh"]
