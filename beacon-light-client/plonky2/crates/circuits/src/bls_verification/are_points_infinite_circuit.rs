use plonky2::{
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};
use plonky2_crypto::biguint::CircuitBuilderBiguint;
use starky_bls12_381::{
    final_exponentiate,
    fp2_plonky2::is_zero,
    fp_plonky2::{is_equal, FpTarget},
    g1_plonky2::PointG1Target,
    g2_plonky2::{signature_point_check, PointG2Target},
};

use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit};

use crate::utils::circuit::verify_proof;

use super::bls12_381_circuit::{
    get_g1_from_miller_loop, get_g2_point_from_pairing_precomp, get_neg_generator,
    BlsCircuitTargets,
};

type F = <C as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

pub struct VerifyIsNotAtInfinityCircuit;

impl Circuit for VerifyIsNotAtInfinityCircuit {
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
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<Self::F, D>,
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
        let pubkey_g1 = get_g1_from_miller_loop(&pt_ml1);
        let is_g1_point_is_at_infinity = is_g1_point_is_at_infinity(builder, &pubkey_g1);
        assert_pk_ne_not_generator(builder, pubkey_g1);

        let signature_g2 = get_g2_point_from_pairing_precomp(builder, &pt_pp2);
        signature_point_check(builder, &signature_g2, &input.sig);
        let is_g2_point_is_at_infinity = is_g2_point_is_at_infinity(builder, &signature_g2);
        assert_g1_or_g2_point_at_infinity(
            builder,
            is_g1_point_is_at_infinity,
            is_g2_point_is_at_infinity,
        );

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

fn assert_g1_or_g2_point_at_infinity(
    builder: &mut CircuitBuilder<F, D>,
    is_g1_at_infinity: BoolTarget,
    is_g2_at_infinity: BoolTarget,
) -> BoolTarget {
    builder.or(is_g1_at_infinity, is_g2_at_infinity)
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
    use std::{fs, marker::PhantomData};

    use anyhow::Result;
    use circuit::Circuit;
    use plonky2::{
        iop::witness::PartialWitness,
        plonk::{
            circuit_data::CircuitData,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

    use crate::bls_verification::bls12_381_circuit::BLSVerificationCircuit;

    type F = <C as GenericConfig<2>>::F;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;
    const SERIALIZED_CIRCUITS_DIR: &str = "../circuit_executables/serialized_circuits";

    fn read_from_file(file_path: &str) -> Result<Vec<u8>> {
        let data = fs::read(file_path)?;
        Ok(data)
    }

    fn load_circuit_data_starky(file_name: &str) -> CircuitData<F, C, D> {
        let circuit_data_bytes = read_from_file(&format!("{file_name}.plonky2_circuit")).unwrap();

        CircuitData::<F, C, D>::from_bytes(
            &circuit_data_bytes,
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_bls12_381_circuit() -> std::result::Result<(), anyhow::Error> {
        let pairing_precomp_circuit_data =
            load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp"));
        let miller_loop_circuit_data =
            load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop"));
        let fp12_mul_circuit_data =
            load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/fp12_mul"));
        let final_exponentiate_circuit_data = load_circuit_data_starky(&format!(
            "{SERIALIZED_CIRCUITS_DIR}/final_exponentiate_circuit"
        ));

        let params = (
            pairing_precomp_circuit_data,
            miller_loop_circuit_data,
            fp12_mul_circuit_data,
            final_exponentiate_circuit_data,
        );

        let (_, circuit) = BLSVerificationCircuit::build(&params);
        let pw = PartialWitness::new();
        let proof = circuit.prove(pw).unwrap();

        circuit.verify(proof)
    }
}
