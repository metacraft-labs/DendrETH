use circuit_derive::{
    AddVirtualTarget, CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, SetWitness,
    TargetPrimitive,
};
use plonky2::{
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};
use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    u32::arithmetic_u32::U32Target,
};
use starky_bls12_381::{
    calc_pairing_precomp,
    fp2_plonky2::is_zero,
    fp_plonky2::{is_equal, FpTarget},
    g1_plonky2::PointG1Target,
    g2_plonky2::PointG2Target,
};

use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit};

use crate::{
    common_targets::{PubkeyTarget, SignatureTarget},
    serializers::serde_bool_array_to_hex_string,
    utils::circuit::bits_to_bytes_target,
};

use super::bls12_381_circuit::{get_neg_generator, N};

type F = <C as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

pub struct VerifyIsNotAtInfinityCircuit;

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct VerifyIsNotAtInfinityCircuitTargets {
    // Pub inputs
    #[target(in)]
    pub validator_credentials: ValidatorCredentials,
}

#[derive(
    Clone,
    Debug,
    TargetPrimitive,
    SetWitness,
    PublicInputsReadable,
    AddVirtualTarget,
    SerdeCircuitTarget,
)]
pub struct ValidatorCredentials {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub signature: SignatureTarget,
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
        let pubkey = input.validator_credentials.pubkey;
        let signature = input.validator_credentials.signature;

        let pubkey = bits_to_bytes_target(builder, &pubkey);
        let pubkey = pubkey.into_iter().collect::<Vec<_>>().try_into().unwrap();
        let pubkey_g1 = get_g1_point_from_public_inputs(&pubkey);
        assert_pk_ne_not_generator(builder, pubkey_g1.to_owned());
        let signature = bits_to_bytes_target(builder, &signature);
        let signature = signature
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let signature_g2 = get_g2_point_from_public_inputs(builder, &signature);
        let is_g1_point_is_at_infinity = is_g1_point_is_at_infinity(builder, &pubkey_g1);
        let is_g2_point_is_at_infinity = is_g2_point_is_at_infinity(builder, &signature_g2);
        assert_g1_or_g2_point_arent_at_infinity(
            builder,
            is_g1_point_is_at_infinity,
            is_g2_point_is_at_infinity,
        );

        let pubkey = input.validator_credentials.pubkey;
        let signature = input.validator_credentials.signature;
        Self::Target {
            validator_credentials: ValidatorCredentials { pubkey, signature },
        }
    }
}

fn assert_pk_ne_not_generator(builder: &mut CircuitBuilder<F, D>, public_key_point: PointG1Target) {
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

fn get_g1_point_from_public_inputs(public_inputs: &[Target; 48]) -> PointG1Target {
    let g1_x = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::X0_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };

    let g1_y = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::X1_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };

    [g1_x, g1_y]
}

fn get_g2_point_from_public_inputs(
    builder: &mut CircuitBuilder<F, D>,
    public_inputs: &[Target; 96],
) -> PointG2Target {
    let sig_point_x0 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::X0_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };
    let sig_point_x1 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::X1_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };
    let sig_point_y0 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::Y0_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };
    let sig_point_y1 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| U32Target(public_inputs[calc_pairing_precomp::Y1_PUBLIC_INPUTS_OFFSET + i]))
            .collect(),
    };

    let zero = builder.zero();
    let one = builder.one();
    for i in 0..N {
        if i == 0 {
            builder.connect(
                public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                one,
            );
        } else {
            builder.connect(
                public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                zero,
            );
        }
        builder.connect(
            public_inputs[calc_pairing_precomp::Z1_PUBLIC_INPUTS_OFFSET + i],
            zero,
        );
    }

    [[sig_point_x0, sig_point_x1], [sig_point_y0, sig_point_y1]]
}

#[cfg(test)]
pub mod tests {
    use circuit::{Circuit, CircuitInput, SetWitness};
    use plonky2::iop::witness::PartialWitness;

    use super::VerifyIsNotAtInfinityCircuit;

    #[test]
    fn test_g1_or_g2_are_at_infinity() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let input =
        serde_json::from_str::<CircuitInput<VerifyIsNotAtInfinityCircuit>>(r#"{
            "validator_credentials": {
                "pubkey": "b301803f8b5ac4a1133581fc676dfedc60d891dd5fa99028805e5ea5b08d3491af75d0707adab3b70c6a6a580217bf81",
                "signature": "b23c46be3a001c63ca711f87a005c200cc550b9429d5f4eb38d74322144f1b63926da3388979e5321012fb1a0526bcd100b5ef5fe72628ce4cd5e904aeaa3279527843fae5ca9ca675f4f51ed8f83bbf7155da9ecc9663100a885d5dc6df96d9"
            }
          }"#).unwrap();

        let mut pw = PartialWitness::new();
        targets.set_witness(&mut pw, &input);

        let proof = circuit.prove(pw).unwrap();
        let _ = circuit.verify(proof);
    }

    #[test]
    #[should_panic]
    fn test_one_privkey() {
        let (targets, circuit) = VerifyIsNotAtInfinityCircuit::build(&());

        let input =
        serde_json::from_str::<CircuitInput<VerifyIsNotAtInfinityCircuit>>(r#"{
            "validator_credentials": {
                "pubkey": "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "signature": "a42ae16f1c2a5fa69c04cb5998d2add790764ce8dd45bf25b29b4700829232052b52352dcff1cf255b3a7810ad7269601810f03b2bc8b68cf289cf295b206770605a190b6842583e47c3d1c0f73c54907bfb2a602157d46a4353a20283018763"
            }
          }"#).unwrap();

        let mut pw = PartialWitness::new();
        targets.set_witness(&mut pw, &input);

        let proof = circuit.prove(pw).unwrap();
        let _ = circuit.verify(proof);
    }
}
