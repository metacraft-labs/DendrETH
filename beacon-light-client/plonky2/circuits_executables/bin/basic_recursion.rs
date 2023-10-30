use std::time::Instant;

use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

fn main() {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;

    let standard_recursion_config = CircuitConfig::standard_recursion_zk_config();
    // standard_recursion_config.zero_knowledge = true;
    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let x = builder.add_virtual_target();
    let a = builder.mul(x, x);
    let b = builder.mul_const(F::from_canonical_u32(4), x);
    let c = builder.mul_const(F::NEG_ONE, b);
    let d = builder.add(a, c);
    let e = builder.add_const(d, F::from_canonical_u32(7));

    // Public inputs are the initial value (provided below) and the result (which is generated).
    builder.register_public_input(x);
    builder.register_public_input(e);
    let data = builder.build::<C>();

    let start = Instant::now();
    let mut pw = PartialWitness::new();
    pw.set_target(x, F::from_canonical_u32(1));
    let proof = data.prove(pw).unwrap();
    println!("Proof time: {:?}", start.elapsed());

    let mut standard_recursion_config = CircuitConfig::standard_recursion_zk_config();
    let mut recursive_builder = CircuitBuilder::<F, D>::new(standard_recursion_config);
    let verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: recursive_builder
            .constant_merkle_cap(&data.verifier_only.constants_sigmas_cap),
        circuit_digest: recursive_builder.constant_hash(data.verifier_only.circuit_digest),
    };
    let pt = recursive_builder.add_virtual_proof_with_pis(&data.common);
    recursive_builder.verify_proof::<C>(&pt, &verifier_circuit_target, &data.common);

    let recursive_data = recursive_builder.build::<C>();

    let start = Instant::now();
    let mut pw = PartialWitness::new();
    pw.set_proof_with_pis_target(&pt, &proof);
    let recursive_proof = recursive_data.prove(pw).unwrap();
    println!("Recursive proof time: {:?}", start.elapsed());
}
