use std::str::FromStr;

use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use num_bigint::BigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    u32::arithmetic_u32::U32Target,
};
use starky_bls12_381::{
    calc_pairing_precomp, final_exponentiate, fp12_mul,
    fp2_plonky2::is_zero,
    fp_plonky2::FpTarget,
    g1_plonky2::{pk_point_check, PointG1Target},
    g2_plonky2::{signature_point_check, PointG2Target},
    hash_to_curve::hash_to_curve,
    miller_loop,
};

use crate::utils::circuit::verify_proof;

const N: usize = 12;

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct BlsCircuitTargets {
    // Pub inputs
    #[target(in, out)]
    pub pubkey: [Target; 48],

    #[target(in, out)]
    pub sig: [Target; 96],

    #[target(in, out)]
    pub msg: [Target; 32],

    #[target(out)]
    pub is_valid_signature: BoolTarget,

    // Proofs
    pub pt_pp1: ProofWithPublicInputsTarget<2>,
    pub pt_pp2: ProofWithPublicInputsTarget<2>,
    pub pt_ml1: ProofWithPublicInputsTarget<2>,
    pub pt_ml2: ProofWithPublicInputsTarget<2>,
    pub pt_fp12m: ProofWithPublicInputsTarget<2>,
    pub pt_fe: ProofWithPublicInputsTarget<2>,
}

type F = <C as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

pub struct BLSVerificationCircuit;

impl Circuit for BLSVerificationCircuit {
    type F = F;
    type C = C;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BlsCircuitTargets;

    type Params = (
        CircuitData<Self::F, Self::C, D>,
        CircuitData<Self::F, Self::C, D>,
        CircuitData<Self::F, Self::C, D>,
        CircuitData<Self::F, Self::C, D>,
    );

    fn define(
        builder: &mut CircuitBuilder<Self::F, D>,
        (
            pairing_precomp_circuit_data,
            miller_loop_circuit_data,
            fp12_mul_circuit_data,
            final_exponentiation_circuit_data,
        ): &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let pt_pp1 = verify_proof(builder, &pairing_precomp_circuit_data);
        let pt_pp2 = verify_proof(builder, &pairing_precomp_circuit_data);
        let pt_ml1 = verify_proof(builder, &miller_loop_circuit_data);
        let pt_ml2 = verify_proof(builder, &miller_loop_circuit_data);
        let pt_fp12m = verify_proof(builder, &fp12_mul_circuit_data);
        let pt_fe = verify_proof(builder, &final_exponentiation_circuit_data);

        let hm = hash_to_curve(builder, &input.msg);

        connect_pairing_precomp_with_g2(builder, &pt_pp1, &hm);

        connect_pairing_precomp_with_miller_loop_g2(builder, &pt_pp1, &pt_ml1);

        let pubkey_g1 = get_g1_from_miller_loop(&pt_ml1);
        pk_point_check(builder, &pubkey_g1, &input.pubkey);
        assert_g1_point_is_at_infinity(builder, &pubkey_g1);

        let signature_g2 = get_g2_point_from_pairing_precomp(builder, &pt_pp2);
        signature_point_check(builder, &signature_g2, &input.sig);
        assert_g2_point_is_at_infinity(builder, &signature_g2);

        connect_pairing_precomp_with_miller_loop_g2(builder, &pt_pp2, &pt_ml2);

        let neg_generator = get_neg_generator(builder);

        connect_miller_loop_with_g1(builder, &neg_generator, &pt_ml2);

        connect_miller_loop_with_fp12_mul(builder, &pt_ml1, &pt_ml2, &pt_fp12m);

        connect_fp12_mull_with_final_exponentiation(builder, &pt_fp12m, &pt_fe);

        let one = builder.one();
        let zero = builder.zero();

        let mut is_valid_signature = builder.is_equal(
            pt_fe.public_inputs[final_exponentiate::PIS_OUTPUT_OFFSET],
            one,
        );

        for i in 1..24 * 3 * 2 {
            let is_valid_signature_i = builder.is_equal(
                pt_fe.public_inputs[final_exponentiate::PIS_OUTPUT_OFFSET + i],
                zero,
            );

            is_valid_signature = builder.and(is_valid_signature, is_valid_signature_i);
        }

        Self::Target {
            pubkey: input.pubkey,
            sig: input.sig,
            msg: input.msg,
            is_valid_signature,
            pt_pp1,
            pt_pp2,
            pt_ml1,
            pt_ml2,
            pt_fp12m,
            pt_fe,
        }
    }
}

fn connect_fp12_mull_with_final_exponentiation<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pt_fp12m: &ProofWithPublicInputsTarget<D>,
    pt_fe: &ProofWithPublicInputsTarget<D>,
) {
    for i in 0..24 * 3 * 2 {
        builder.connect(
            pt_fp12m.public_inputs[fp12_mul::PIS_OUTPUT_OFFSET + i],
            pt_fe.public_inputs[final_exponentiate::PIS_INPUT_OFFSET + i],
        );
    }
}

fn get_neg_generator<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> PointG1Target {
    let neg_generator_x = builder.constant_biguint(&BigUint::from_str("3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507").unwrap());
    let neg_generator_y = builder.constant_biguint(&BigUint::from_str("2662903010277190920397318445793982934971948944000658264905514399707520226534504357969962973775649129045502516118218").unwrap());

    [neg_generator_x, neg_generator_y]
}

fn get_g2_point_from_pairing_precomp<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pt_pp2: &ProofWithPublicInputsTarget<D>,
) -> PointG2Target {
    let sig_point_x0 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_pp2.public_inputs[calc_pairing_precomp::X0_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };
    let sig_point_x1 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_pp2.public_inputs[calc_pairing_precomp::X1_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };
    let sig_point_y0 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_pp2.public_inputs[calc_pairing_precomp::Y0_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };
    let sig_point_y1 = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_pp2.public_inputs[calc_pairing_precomp::Y1_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };

    let zero = builder.zero();
    let one = builder.one();
    for i in 0..N {
        if i == 0 {
            builder.connect(
                pt_pp2.public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                one,
            );
        } else {
            builder.connect(
                pt_pp2.public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                zero,
            );
        }
        builder.connect(
            pt_pp2.public_inputs[calc_pairing_precomp::Z1_PUBLIC_INPUTS_OFFSET + i],
            zero,
        );
    }

    [[sig_point_x0, sig_point_x1], [sig_point_y0, sig_point_y1]]
}

fn connect_miller_loop_with_fp12_mul<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pt_ml1: &ProofWithPublicInputsTarget<D>,
    pt_ml2: &ProofWithPublicInputsTarget<D>,
    pt_fp12m: &ProofWithPublicInputsTarget<D>,
) {
    for i in 0..24 * 3 * 2 {
        builder.connect(
            pt_ml1.public_inputs[miller_loop::PIS_RES_OFFSET + i],
            pt_fp12m.public_inputs[fp12_mul::PIS_INPUT_X_OFFSET + i],
        );
    }

    for i in 0..24 * 3 * 2 {
        builder.connect(
            pt_ml2.public_inputs[miller_loop::PIS_RES_OFFSET + i],
            pt_fp12m.public_inputs[fp12_mul::PIS_INPUT_Y_OFFSET + i],
        );
    }
}

fn connect_miller_loop_with_g1<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pubkey_g1: &PointG1Target,
    pt_ml1: &ProofWithPublicInputsTarget<D>,
) {
    for i in 0..12 {
        builder.connect(
            pubkey_g1[0].limbs[i].0,
            pt_ml1.public_inputs[miller_loop::PIS_PX_OFFSET + i],
        );

        builder.connect(
            pubkey_g1[1].limbs[i].0,
            pt_ml1.public_inputs[miller_loop::PIS_PY_OFFSET + i],
        );
    }
}

fn get_g1_from_miller_loop(pt_ml1: &ProofWithPublicInputsTarget<D>) -> PointG1Target {
    let g1_x = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_ml1.public_inputs[calc_pairing_precomp::X0_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };

    let g1_y = BigUintTarget {
        limbs: (0..N)
            .into_iter()
            .map(|i| {
                U32Target(pt_ml1.public_inputs[calc_pairing_precomp::X1_PUBLIC_INPUTS_OFFSET + i])
            })
            .collect(),
    };

    [g1_x, g1_y]
}

fn connect_pairing_precomp_with_miller_loop_g2<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pt_pp1: &plonky2::plonk::proof::ProofWithPublicInputsTarget<D>,
    pt_ml1: &plonky2::plonk::proof::ProofWithPublicInputsTarget<D>,
) {
    for i in 0..68 * 3 * 24 {
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::ELL_COEFFS_PUBLIC_INPUTS_OFFSET + i],
            pt_ml1.public_inputs[miller_loop::PIS_ELL_COEFFS_OFFSET + i],
        );
    }
}

fn connect_pairing_precomp_with_g2<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pt_pp1: &plonky2::plonk::proof::ProofWithPublicInputsTarget<D>,
    hm: &PointG2Target,
) {
    let zero = builder.zero();
    let one = builder.one();
    for i in 0..N {
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::X0_PUBLIC_INPUTS_OFFSET + i],
            hm[0][0].limbs.get(i).unwrap_or(&U32Target(zero)).0,
        );
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::X1_PUBLIC_INPUTS_OFFSET + i],
            hm[0][1].limbs.get(i).unwrap_or(&U32Target(zero)).0,
        );
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::Y0_PUBLIC_INPUTS_OFFSET + i],
            hm[1][0].limbs.get(i).unwrap_or(&U32Target(zero)).0,
        );
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::Y1_PUBLIC_INPUTS_OFFSET + i],
            hm[1][1].limbs.get(i).unwrap_or(&U32Target(zero)).0,
        );
        builder.connect(
            pt_pp1.public_inputs[calc_pairing_precomp::Z1_PUBLIC_INPUTS_OFFSET + i],
            zero,
        );

        if i == 0 {
            builder.connect(
                pt_pp1.public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                one,
            );
        } else {
            builder.connect(
                pt_pp1.public_inputs[calc_pairing_precomp::Z0_PUBLIC_INPUTS_OFFSET + i],
                zero,
            );
        }
    }
}

fn assert_g1_point_is_at_infinity(builder: &mut CircuitBuilder<F, D>, g1_point: &PointG1Target) {
    let is_g1_x_zero = is_fp_zero(builder, &g1_point[0]);
    let is_g1_y_zero = is_fp_zero(builder, &g1_point[1]);
    let are_g1_x_and_y_zero = builder.and(is_g1_x_zero, is_g1_y_zero);
    builder.assert_true(are_g1_x_and_y_zero)
}

fn assert_g2_point_is_at_infinity(builder: &mut CircuitBuilder<F, D>, g2_point: &PointG2Target) {
    let is_g2_x_zero = is_zero(builder, &g2_point[0]);
    let is_g2_y_zero = is_zero(builder, &g2_point[1]);
    let are_g2_x_and_y_zero = builder.and(is_g2_x_zero, is_g2_y_zero);
    builder.assert_true(are_g2_x_and_y_zero)
}

fn is_fp_zero<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    input: &FpTarget,
) -> BoolTarget {
    let zero = builder.zero_biguint();
    builder.cmp_biguint(input, &zero)
}

#[cfg(test)]
pub mod tests {
    use circuit::Circuit;
    use plonky2::plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    };

    use super::BLSVerificationCircuit;

    type F = <C as GenericConfig<2>>::F;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    #[test]
    fn test_bls12_381_circuit() {
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);
        // BLSVerificationCircuit::define(&mut builder, params);
    }
}
