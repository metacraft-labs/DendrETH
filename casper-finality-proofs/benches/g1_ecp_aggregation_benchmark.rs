use casper_finality_proofs::verification::curves::g1::g1_ecc_aggregate;
use casper_finality_proofs::verification::pubkey_to_g1::pubkey_to_g1_check;
use num_bigint::BigUint;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::plonk::config::GenericConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use plonky2x::frontend::uint::num::biguint::CircuitBuilderBiguint;

fn g1_ecp_aggregation_benchmark(c: &mut Criterion) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let circuit_config = plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();
    let mut builder = plonky2::plonk::circuit_builder::CircuitBuilder::<F, D>::new(circuit_config);

    let a_bigu = BigUint::from_str(
        "1216495682195235861952885506871698490232894470117269383940381148575524314493849307811227440691167647909822763414941"
    ).unwrap();
    let b_bigu = BigUint::from_str(
        "2153848155426317245700560287567131132765685008362732985860101000686875894603366983854567186180519945327668975076337"
    ).unwrap();

    let g1_a = black_box([
        builder.constant_biguint(&a_bigu),
        builder.constant_biguint(&b_bigu),
    ]);

    let g1_b = black_box([
        builder.constant_biguint(&a_bigu),
        builder.constant_biguint(&b_bigu),
    ]);

    let point = g1_ecc_aggregate(&mut builder, g1_a.clone(), g1_b.clone());

    let pk: Vec<Target> = [
        137, 43, 218, 171, 28, 7, 187, 176, 109, 242, 254, 250, 130, 131, 36, 52, 5, 250, 52, 180,
        134, 10, 178, 231, 178, 58, 55, 126, 255, 212, 103, 96, 128, 72, 218, 203, 176, 158, 145,
        7, 181, 216, 163, 154, 82, 112, 159, 221,
    ]
    .iter()
    .map(|f| builder.constant(GoldilocksField::from_canonical_u8(*f)))
    .collect();

    let pk: [Target; 48] = pk.into_iter().collect::<Vec<Target>>().try_into().unwrap();

    c.bench_function("aggregation of g1 points on EC", |b| {
        b.iter(|| pubkey_to_g1_check(&mut builder, &[point[0].clone(), point[1].clone()], &pk))
    });
}

criterion_group!(benches, g1_ecp_aggregation_benchmark);
criterion_main!(benches);
