use std::marker::PhantomData;

use circuit::{Circuit, SerdeCircuitTarget};
use circuit_executables::{
    crud::common::{load_circuit_data_starky, write_to_file},
    utils::CommandLineOptionsBuilder,
};
use circuits::bls_verification::bls12_381_circuit::BLSVerificationCircuit;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

const CIRCUIT_NAME: &str = "bls12_381";

fn main() {
    let matches = CommandLineOptionsBuilder::new("bls12_381_circuit_data_generation")
        .with_serialized_circuits_dir()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/pairing_precomp"));
    let miller_loop_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/miller_loop"));
    let fp12_mul_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/fp12_mul"));
    let final_exponentiate_circuit_data = load_circuit_data_starky(&format!(
        "{serialized_circuits_dir}/final_exponentiate_circuit"
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
            serialized_circuits_dir, CIRCUIT_NAME
        ),
        &circuit_bytes,
    )
    .unwrap();

    let target_bytes = targets.serialize().unwrap();

    write_to_file(
        &format!(
            "{}/{}.plonky2_targets",
            serialized_circuits_dir, CIRCUIT_NAME
        ),
        &target_bytes,
    )
    .unwrap();
}
