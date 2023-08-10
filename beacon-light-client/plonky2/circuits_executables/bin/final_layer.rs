use std::fs;

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::fri::reduction_strategies::FriReductionStrategy;
use plonky2::fri::FriConfig;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

fn main() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_config = CircuitConfig::standard_recursion_config();

    // A high-rate recursive proof, designed to be verifiable with fewer routed wires.
    let high_rate_config = CircuitConfig {
        fri_config: FriConfig {
            rate_bits: 7,
            proof_of_work_bits: 16,
            num_query_rounds: 12,
            ..standard_config.fri_config.clone()
        },
        ..standard_config
    };

    // A final proof, optimized for size.
    let final_config = CircuitConfig {
        num_routed_wires: 37,
        fri_config: FriConfig {
            rate_bits: 8,
            cap_height: 0,
            proof_of_work_bits: 20,
            reduction_strategy: FriReductionStrategy::MinSize(None),
            num_query_rounds: 10,
        },
        ..high_rate_config
    };
    let mut builder = CircuitBuilder::<GoldilocksField, D>::new(final_config.clone());

    // The arithmetic circuit.
    let x = builder.add_virtual_target();
    let a = builder.mul(x, x);
    let b = builder.mul_const(F::from_canonical_u32(4), x);
    let c = builder.mul_const(F::NEG_ONE, b);
    let d = builder.add(a, c);
    let e = builder.add_const(d, F::from_canonical_u32(7));

    // Public inputs are the initial value (provided below) and the result (which is generated).
    builder.register_public_input(x);
    builder.register_public_input(e);
    let mut pw = PartialWitness::new();
    pw.set_target(x, F::from_canonical_u32(1));
    let data = builder.build::<C>();
    let proof = data.prove(pw)?;

    println!(
        "x² - 4 *x + 7 where x = {} is {}",
        proof.public_inputs[0], proof.public_inputs[1]
    );

    fs::write(
        "common_circuit_data.json",
        serde_json::to_string(&data.common)?,
    )?;

    fs::write(
        "verifier_only_circuit_data.json",
        serde_json::to_string(&data.verifier_only)?,
    )?;

    fs::write(
        "proof_with_public_inputs.json",
        serde_json::to_string(&proof)?,
    )?;

    data.verify(proof)
}
