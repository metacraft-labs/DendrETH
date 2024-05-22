use std::marker::PhantomData;

use circuit_executables::crud::common::{load_circuit_data_starky, read_from_file, write_to_file};
use circuits::{
    bls12_381_circuit::{build_bls12_381_circuit, BlsCircuitTargets},
    serialization::targets_serialization::WriteTargets,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "bls12_381";

fn main() {
    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{CIRCUIT_DIR}/pairing_precomp"));
    let miller_loop_circuit_data = load_circuit_data_starky(&format!("{CIRCUIT_DIR}/miller_loop"));
    let fp12_mul_circuit_data = load_circuit_data_starky(&format!("{CIRCUIT_DIR}/fp12_mul"));
    let final_exponentiate_circuit_data =
        load_circuit_data_starky(&format!("{CIRCUIT_DIR}/final_exponentiate_circuit"));

    let (targets, data): (
        BlsCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ) = build_bls12_381_circuit(
        &pairing_precomp_circuit_data,
        &miller_loop_circuit_data,
        &fp12_mul_circuit_data,
        &final_exponentiate_circuit_data,
    );

    let circuit_bytes = data
        .to_bytes(
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            },
        )
        .unwrap();

    write_to_file(
        &format!("{}/{}.plonky2_circuit", CIRCUIT_DIR, CIRCUIT_NAME),
        &circuit_bytes,
    )
    .unwrap();

    let target_bytes = targets.write_targets().unwrap();

    write_to_file(
        &format!("{}/{}.plonky2_targets", CIRCUIT_DIR, CIRCUIT_NAME),
        &target_bytes,
    )
    .unwrap();
}
