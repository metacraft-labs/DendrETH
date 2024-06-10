use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
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

use super::bls12_381_circuit::{get_neg_generator, N};

type F = <C as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

pub struct VerifyIsNotAtInfinityCircuit;

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct VerifyIsNotAtInfinityCircuitTargets {
    // Pub inputs
    #[target(in, out)]
    pub pubkey: [Target; 48],

    #[target(in, out)]
    pub sig: [Target; 96],

    #[target(out)]
    pub is_at_infinity: BoolTarget,
}

impl Circuit for VerifyIsNotAtInfinityCircuit {
    type F = F;
    type C = C;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();
    type Target = VerifyIsNotAtInfinityCircuitTargets;

    type Params = ([Target; 48], [Target; 96]);

    fn define(
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<Self::F, D>,
        params: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);
        let pubkey = params.0;
        let signature = params.1;

        let pubkey_g1 = get_g1_point_from_public_inputs(&pubkey);
        assert_pk_ne_not_generator(builder, pubkey_g1.to_owned());

        let signature_g2 = get_g2_point_from_public_inputs(builder, &signature);
        let is_g1_point_is_at_infinity = is_g1_point_is_at_infinity(builder, &pubkey_g1);
        let is_g2_point_is_at_infinity = is_g2_point_is_at_infinity(builder, &signature_g2);
        assert_g1_or_g2_point_at_infinity(
            builder,
            is_g1_point_is_at_infinity,
            is_g2_point_is_at_infinity,
        );

        Self::Target {
            pubkey: input.pubkey,
            sig: input.sig,
            is_at_infinity: builder._false(),
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
    use std::{fs, marker::PhantomData};

    use anyhow::Result;
    use circuit::Circuit;
    use plonky2::{
        iop::{target::Target, witness::PartialWitness},
        plonk::{
            circuit_data::CircuitData,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

    use super::VerifyIsNotAtInfinityCircuit;

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

        let params = (miller_loop_circuit_data, pairing_precomp_circuit_data);
        let miller_loop_public_inputs: [Target; 48] = params
            .0
            .prover_only
            .public_inputs
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let pairing_precomp_public_inputs: [Target; 96] = params
            .1
            .prover_only
            .public_inputs
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let (_, circuit) = VerifyIsNotAtInfinityCircuit::build(&(
            miller_loop_public_inputs,
            pairing_precomp_public_inputs,
        ));
        let pw = PartialWitness::new();
        let proof = circuit.prove(pw).unwrap();

        circuit.verify(proof)
    }
}
