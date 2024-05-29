use std::marker::PhantomData;

use circuit::{Circuit, SerdeCircuitTarget};
use circuit_executables::{
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::common::{load_circuit_data_starky, write_to_file},
};
use circuits::bls_verification::bls12_381_circuit::{BLSVerificationCircuit, BlsCircuitTargets};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

const CIRCUIT_NAME: &str = "bls12_381";

fn main() {
    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp"));
    let miller_loop_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop"));
    let fp12_mul_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/fp12_mul"));
    let final_exponentiate_circuit_data = load_circuit_data_starky(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/final_exponentiate_circuit"
    ));

    let (targets, data) = BLSVerificationCircuit::build(&(
        pairing_precomp_circuit_data,
        miller_loop_circuit_data,
        fp12_mul_circuit_data,
        final_exponentiate_circuit_data,
    ));

    let circuit_bytes = data
        .to_bytes(
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            },
        )
        .unwrap();

    write_to_file(
        &format!(
            "{}/{}.plonky2_circuit",
            SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME
        ),
        &circuit_bytes,
    )
    .unwrap();

    let target_bytes = targets.serialize().unwrap();

    write_to_file(
        &format!(
            "{}/{}.plonky2_targets",
            SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME
        ),
        &target_bytes,
    )
    .unwrap();
}
