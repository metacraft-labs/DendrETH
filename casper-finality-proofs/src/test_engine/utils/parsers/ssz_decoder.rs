use lighthouse_types::{BeaconState, ChainSpec, EthSpec};
use snap::raw::Decoder;
use std::fs::{self};

pub fn read_ssz_fixture<E: EthSpec>(path: &str, spec: &ChainSpec) -> BeaconState<E> {
    let compressed_bytes = fs::read(path).unwrap();
    let mut decoder = Decoder::new();
    let ssz_bytes = decoder.decompress_vec(&compressed_bytes).unwrap();
    BeaconState::from_ssz_bytes(ssz_bytes.as_slice(), &spec).unwrap()
}
