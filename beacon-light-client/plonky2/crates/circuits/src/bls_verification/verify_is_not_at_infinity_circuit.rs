use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
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

use super::bls12_381_circuit::get_neg_generator;

type F = <C as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

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
        assert_pk_ne_not_generator(builder, &pubkey_g1);
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

fn assert_pk_ne_not_generator(
    builder: &mut CircuitBuilder<F, D>,
    public_key_point: &PointG1Target,
) {
    let get_neg_generator = get_neg_generator(builder);

    let is_pk_point_x_eq_not_generator_x =
        is_equal(builder, &public_key_point[0], &get_neg_generator[0]);
    let is_pk_point_y_eq_not_generator_y =
        is_equal(builder, &public_key_point[1], &get_neg_generator[1]);
    let pk_point_eq_not_generator = builder.and(
        is_pk_point_x_eq_not_generator_x,
        is_pk_point_y_eq_not_generator_y,
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
    use plonky2::iop::witness::PartialWitness;
    use plonky2_crypto::biguint::WitnessBigUint;

    use super::VerifyIsNotAtInfinityCircuit;

    #[test]
    fn test_g1_or_g2_are_at_infinity() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let pubkey = "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73";
        println!("pubkey_as_bytes are: {:?}", pubkey.as_bytes());
        let signature = "b735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275";
        println!("signature_as_bytes are: {:?}", signature.as_bytes());

        let input = serde_json::from_str::<CircuitInput<VerifyIsNotAtInfinityCircuit>>(
            r#"{
            "pubkey_bytes": [
                98, 55, 56, 49, 57, 53, 54, 49, 49, 48, 100, 50, 52, 101, 52, 53, 49, 48, 97, 56, 98,
                53, 53, 48, 48, 98, 55, 49, 53, 50, 57, 102, 56, 54, 51, 53, 97, 97, 52, 49, 57, 97,
                48, 48, 57, 100, 51, 49, 52, 56, 57, 56, 101, 56, 99, 53, 55, 50, 97, 52, 102, 57, 50,
                51, 98, 97, 54, 52, 51, 97, 101, 57, 52, 98, 100, 102, 100, 102, 57, 50, 50, 52, 53,
                48, 57, 49, 55, 55, 97, 97, 56, 101, 54, 98, 55, 51
            ],
            "sig_bytes": [
                98, 55, 51, 53, 100, 48, 100, 48, 98, 48, 51, 102, 53, 49, 102, 99, 102, 51, 101, 53,
                98, 99, 53, 49, 48, 98, 53, 97, 50, 99, 98, 50, 54, 54, 48, 55, 53, 51, 50, 50, 102,
                53, 55, 54, 49, 97, 54, 57, 53, 52, 55, 55, 56, 55, 49, 52, 102, 53, 97, 98, 56, 56,
                51, 49, 98, 99, 57, 57, 52, 53, 52, 51, 56, 48, 100, 51, 51, 48, 102, 53, 99, 49, 57,
                100, 57, 51, 52, 51, 54, 102, 48, 99, 52, 51, 51, 57, 48, 52, 49, 98, 102, 101, 101,
                99, 100, 50, 49, 54, 49, 97, 49, 50, 50, 99, 49, 99, 101, 56, 52, 50, 56, 48, 51, 51,
                100, 98, 56, 100, 100, 97, 49, 52, 50, 55, 54, 56, 97, 52, 56, 101, 53, 56, 50, 102,
                53, 102, 57, 98, 100, 101, 55, 100, 52, 48, 55, 54, 56, 97, 99, 53, 97, 51, 98, 54, 97,
                56, 48, 52, 57, 50, 98, 55, 51, 55, 49, 57, 102, 49, 53, 50, 51, 99, 53, 100, 97, 51,
                53, 100, 101, 50, 55, 53
            ]
          }"#,
        )
        .unwrap();

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
}
