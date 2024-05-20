use casper_finality_proofs::compute_shuffled_index::circuit::define;
use plonky2x::prelude::{
    bytes, ArrayVariable, ByteVariable, CircuitBuilder, DefaultParameters, U64Variable,
};
use plonky2x::utils;

fn main() {
    utils::setup_logger();

    let seed_bytes: Vec<u8> =
        bytes!("0x4ac96f664a6cafd300b161720809b9e17905d4d8fed7a97ff89cf0080a953fe7");

    let seed_bytes_fixed_size: [u8; 32] = seed_bytes.try_into().unwrap();

    const SHUFFLE_ROUND_COUNT: u8 = 90;
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();

    define(&mut builder, SHUFFLE_ROUND_COUNT);

    let circuit = builder.mock_build();

    const START_IDX: u64 = 0;
    const COUNT: u64 = 100;
    let mapping = [
        53, 21, 19, 29, 76, 32, 67, 63, 3, 38, 89, 37, 30, 78, 0, 40, 96, 44, 22, 42, 23, 62, 92,
        87, 11, 43, 54, 75, 71, 82, 68, 36, 59, 90, 66, 45, 58, 70, 4, 72, 33, 24, 6, 39, 52, 51,
        99, 8, 27, 88, 20, 31, 86, 77, 94, 95, 85, 41, 93, 15, 13, 5, 74, 81, 18, 17, 47, 2, 16, 7,
        84, 9, 79, 65, 61, 49, 60, 50, 64, 34, 55, 56, 91, 98, 28, 46, 14, 73, 12, 25, 26, 57, 83,
        80, 35, 97, 69, 10, 1, 48,
    ];
    for i in START_IDX..COUNT {
        let mut input = circuit.input();

        input.write::<U64Variable>(i);
        input.write::<U64Variable>(COUNT);
        input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

        let (_witness, mut _output) = circuit.mock_prove(&input);
        let shuffled_index_res = _output.read::<U64Variable>();

        println!("{} {}", mapping[i as usize], shuffled_index_res);
        assert!(mapping[i as usize] == shuffled_index_res);
    }
}
