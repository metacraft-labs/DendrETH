use ark_bls12_381::g1::{G1_GENERATOR_X, G1_GENERATOR_Y};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use num::BigUint;
use plonky2::{
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};
use starky_bls12_381::{
    fp2_plonky2::is_zero,
    fp_plonky2::{is_equal, FpTarget},
    g1_plonky2::{pk_point_check, PointG1Target},
    g2_plonky2::{signature_point_check, PointG2Target},
};

use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit};

pub type F = <C as GenericConfig<2>>::F;
pub type C = PoseidonGoldilocksConfig;
pub const D: usize = 2;

pub struct VerifyIsNotAtInfinityCircuit;

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct VerifyIsNotAtInfinityCircuitTargets {
    // Pub inputs
    #[target(in, out)]
    pub pubkey_bytes: [Target; 48],

    #[target(in, out)]
    pub sig_bytes: [Target; 96],

    pub pubkey_g1_x: BigUintTarget,
    pub pubkey_g1_y: BigUintTarget,
    pub sig_g2_x_c0: BigUintTarget,
    pub sig_g2_x_c1: BigUintTarget,
    pub sig_g2_y_c0: BigUintTarget,
    pub sig_g2_y_c1: BigUintTarget,
}

impl Circuit for VerifyIsNotAtInfinityCircuit {
    type F = F;
    type C = C;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();
    type Target = VerifyIsNotAtInfinityCircuitTargets;

    fn define(
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<Self::F, D>,
        _: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let pubkey_g1_x = builder.add_virtual_biguint_target(12);
        let pubkey_g1_y = builder.add_virtual_biguint_target(12);

        let sig_g2_x_c0 = builder.add_virtual_biguint_target(12);
        let sig_g2_x_c1 = builder.add_virtual_biguint_target(12);
        let sig_g2_y_c0 = builder.add_virtual_biguint_target(12);
        let sig_g2_y_c1 = builder.add_virtual_biguint_target(12);
        let pubkey_bytes = input.pubkey_bytes;

        let pubkey_g1 = [pubkey_g1_x.to_owned(), pubkey_g1_y.to_owned()];
        pk_point_check(builder, &pubkey_g1, &pubkey_bytes);
        assert_pk_ne_g1_generator(builder, &pubkey_g1);
        let is_g1_point_is_at_infinity = is_g1_point_is_at_infinity(builder, &pubkey_g1);

        let sig_bytes = input.sig_bytes;
        let sig_g2 = [
            [sig_g2_x_c0.to_owned(), sig_g2_x_c1.to_owned()],
            [sig_g2_y_c0.to_owned(), sig_g2_y_c1.to_owned()],
        ];
        signature_point_check(builder, &sig_g2, &sig_bytes);
        let is_g2_point_is_at_infinity = is_g2_point_is_at_infinity(builder, &sig_g2);

        assert_g1_or_g2_point_arent_at_infinity(
            builder,
            is_g1_point_is_at_infinity,
            is_g2_point_is_at_infinity,
        );

        Self::Target {
            pubkey_bytes,
            sig_bytes,
            pubkey_g1_x,
            pubkey_g1_y,
            sig_g2_x_c0,
            sig_g2_x_c1,
            sig_g2_y_c0,
            sig_g2_y_c1,
        }
    }
}

fn assert_pk_ne_g1_generator(builder: &mut CircuitBuilder<F, D>, public_key_point: &PointG1Target) {
    let g1_generator_x = builder.constant_biguint(&BigUint::from(G1_GENERATOR_X));
    let g1_generator_y = builder.constant_biguint(&BigUint::from(G1_GENERATOR_Y));

    let is_pk_point_x_eq_g1_generator_x = is_equal(builder, &public_key_point[0], &g1_generator_x);
    let is_pk_point_y_eq_g1_generator_y = is_equal(builder, &public_key_point[1], &g1_generator_y);
    let pk_point_eq_not_generator = builder.and(
        is_pk_point_x_eq_g1_generator_x,
        is_pk_point_y_eq_g1_generator_y,
    );
    builder.assert_false(pk_point_eq_not_generator)
}

fn assert_g1_or_g2_point_arent_at_infinity(
    builder: &mut CircuitBuilder<F, D>,
    is_g1_at_infinity: BoolTarget,
    is_g2_at_infinity: BoolTarget,
) {
    let g1_or_g2_point_is_at_infinity = builder.or(is_g1_at_infinity, is_g2_at_infinity);
    builder.assert_false(g1_or_g2_point_is_at_infinity);
}

fn is_g1_point_is_at_infinity(
    builder: &mut CircuitBuilder<F, D>,
    g1_point: &PointG1Target,
) -> BoolTarget {
    let is_g1_x_zero = is_fp_zero(builder, &g1_point[0]);
    let is_g1_y_zero = is_fp_zero(builder, &g1_point[1]);
    builder.and(is_g1_x_zero, is_g1_y_zero)
}

fn is_g2_point_is_at_infinity(
    builder: &mut CircuitBuilder<F, D>,
    g2_point: &PointG2Target,
) -> BoolTarget {
    let is_g2_x_zero = is_zero(builder, &g2_point[0]);
    let is_g2_y_zero = is_zero(builder, &g2_point[1]);
    builder.and(is_g2_x_zero, is_g2_y_zero)
}

fn is_fp_zero(builder: &mut CircuitBuilder<F, D>, input: &FpTarget) -> BoolTarget {
    let zero = builder.zero_biguint();
    builder.cmp_biguint(input, &zero)
}

#[cfg(test)]
pub mod tests {
    use ark_bls12_381::{G1Affine, G2Affine};
    use ark_serialize::CanonicalDeserialize;
    use circuit::{Circuit, CircuitInput, SetWitness};
    use num::BigUint;
    use plonky2::{iop::witness::PartialWitness, plonk::circuit_data::CircuitData};
    use plonky2_crypto::biguint::WitnessBigUint;

    use super::{
        VerifyIsNotAtInfinityCircuit, VerifyIsNotAtInfinityCircuitTargets,
        VerifyIsNotAtInfinityCircuitTargetsWitnessInput, C, D, F,
    };

    fn input_init(
        pubkey: &str,
        signature: &str,
    ) -> VerifyIsNotAtInfinityCircuitTargetsWitnessInput {
        let pubkey_bytes = hex::decode(pubkey).unwrap();
        let pubkey_bytes: [u8; 48] = pubkey_bytes
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let sig_bytes = hex::decode(signature).unwrap();
        let sig_bytes: [u8; 96] = sig_bytes
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let concatenated_str = format!(
            r#"{{
                "pubkey_bytes": {:?},
                "sig_bytes": {:?}
            }}"#,
            pubkey_bytes, sig_bytes
        );

        serde_json::from_str::<CircuitInput<VerifyIsNotAtInfinityCircuit>>(&concatenated_str)
            .unwrap()
    }

    fn test_helper(
        pubkey: &str,
        signature: &str,
        targets: VerifyIsNotAtInfinityCircuitTargets,
        circuit: CircuitData<F, C, D>,
        input: VerifyIsNotAtInfinityCircuitTargetsWitnessInput,
    ) {
        let pubkey_g1: G1Affine =
            G1Affine::deserialize_compressed_unchecked(&*hex::decode(pubkey).unwrap()).unwrap();
        let signature_g2 =
            G2Affine::deserialize_compressed_unchecked(&*hex::decode(signature).unwrap()).unwrap();

        let mut pw = PartialWitness::new();
        targets.set_witness(&mut pw, &input);
        pw.set_biguint_target(
            &targets.pubkey_g1_x,
            &BigUint::try_from(pubkey_g1.x).unwrap(),
        );
        pw.set_biguint_target(
            &targets.pubkey_g1_y,
            &BigUint::try_from(pubkey_g1.y).unwrap(),
        );
        pw.set_biguint_target(
            &targets.sig_g2_x_c0,
            &BigUint::try_from(signature_g2.x.c0).unwrap(),
        );
        pw.set_biguint_target(
            &targets.sig_g2_x_c1,
            &BigUint::try_from(signature_g2.x.c1).unwrap(),
        );
        pw.set_biguint_target(
            &targets.sig_g2_y_c0,
            &BigUint::try_from(signature_g2.y.c0).unwrap(),
        );
        pw.set_biguint_target(
            &targets.sig_g2_y_c1,
            &BigUint::try_from(signature_g2.y.c1).unwrap(),
        );

        let proof = circuit.prove(pw).unwrap();
        let _ = circuit.verify(proof);
    }

    #[test]
    fn test_valid_case_for_g1_and_g2_at_infinity() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let pubkey = "b301803f8b5ac4a1133581fc676dfedc60d891dd5fa99028805e5ea5b08d3491af75d0707adab3b70c6a6a580217bf81";
        let signature = "b23c46be3a001c63ca711f87a005c200cc550b9429d5f4eb38d74322144f1b63926da3388979e5321012fb1a0526bcd100b5ef5fe72628ce4cd5e904aeaa3279527843fae5ca9ca675f4f51ed8f83bbf7155da9ecc9663100a885d5dc6df96d9";
        let input = input_init(pubkey, signature);

        test_helper(pubkey, signature, targets, circuit, input);
    }

    #[test]
    #[should_panic]
    fn test_g1_or_g2_are_at_infinity() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let pubkey = "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let signature = "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let input = input_init(pubkey, signature);

        test_helper(pubkey, signature, targets, circuit, input);
    }

    #[test]
    #[should_panic]
    fn test_g1_is_not_the_g1_generator() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let pubkey = "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb";
        let signature = "a42ae16f1c2a5fa69c04cb5998d2add790764ce8dd45bf25b29b4700829232052b52352dcff1cf255b3a7810ad7269601810f03b2bc8b68cf289cf295b206770605a190b6842583e47c3d1c0f73c54907bfb2a602157d46a4353a20283018763";
        let input = input_init(pubkey, signature);

        test_helper(pubkey, signature, targets, circuit, input);
    }
}
