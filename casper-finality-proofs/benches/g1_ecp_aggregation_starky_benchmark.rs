use ark_std::iterable::Iterable;
use casper_finality_proofs::verification::pubkey_to_g1::pubkey_to_g1_check;
use casper_finality_proofs::verification::utils::native_bls::Fp;
use casper_finality_proofs::verification::verify::verify_pubkeys_aggregation;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use num_bigint::BigUint;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::plonk::config::GenericConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2x::frontend::uint::num::biguint::CircuitBuilderBiguint;
use std::str::FromStr;

fn g1_ecp_aggregation_starky_benchmark(c: &mut Criterion) {
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

    let a_fp = black_box(Fp::get_fp_from_biguint(a_bigu.clone()));
    let b_fp = black_box(Fp::get_fp_from_biguint(b_bigu.clone()));

    let ec_proof = verify_pubkeys_aggregation(vec![[a_fp, b_fp]], [a_fp, b_fp], vec![true]);

    let ec_proof_pub_inputs = ec_proof.0.public_inputs;
    for i in 0..12 {
        println!("g1_x_input is: {:?}", ec_proof_pub_inputs[i].0)
    }

    let g1_x_input = builder.constant_biguint(&BigUint::new(
        ec_proof_pub_inputs[0..12]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g1_y_input = builder.constant_biguint(&BigUint::new(
        ec_proof_pub_inputs[12..24]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    let pk: Vec<Target> = [
        137, 43, 218, 171, 28, 7, 187, 176, 109, 242, 254, 250, 130, 131, 36, 52, 5, 250, 52, 180,
        134, 10, 178, 231, 178, 58, 55, 126, 255, 212, 103, 96, 128, 72, 218, 203, 176, 158, 145,
        7, 181, 216, 163, 154, 82, 112, 159, 221,
    ]
    .iter()
    .map(|f| builder.constant(GoldilocksField::from_canonical_u8(f)))
    .collect();

    let pk: [Target; 48] = pk.into_iter().collect::<Vec<Target>>().try_into().unwrap();

    c.bench_function("aggregation of g1 points on EC with stark", |b| {
        b.iter(|| pubkey_to_g1_check(&mut builder, &[g1_x_input.clone(), g1_y_input.clone()], &pk))
    });
}

criterion_group!(benches, g1_ecp_aggregation_starky_benchmark);
criterion_main!(benches);
