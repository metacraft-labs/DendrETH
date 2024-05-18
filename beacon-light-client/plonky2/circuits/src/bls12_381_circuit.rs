use std::str::FromStr;

use num_bigint::BigUint;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Read, Write},
};
use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    u32::arithmetic_u32::U32Target,
};
use starky_bls12_381::{
    calc_pairing_precomp, final_exponentiate, fp12_mul,
    g1_plonky2::{pk_point_check, PointG1Target},
    g2_plonky2::{signature_point_check, PointG2Target},
    hash_to_curve::hash_to_curve,
    miller_loop,
};

use crate::targets_serialization::{ReadTargets, WriteTargets};

const N: usize = 12;

pub struct BlsCircuitTargets {
    // Pub inputs
    pub pubkey: [Target; 48],
    pub sig: [Target; 96],
    pub msg: [Target; 32],
    pub is_valid_signature: Target,

    // Proofs
    pub pt_pp1: ProofWithPublicInputsTarget<2>,
    pub pt_pp2: ProofWithPublicInputsTarget<2>,
    pub pt_ml1: ProofWithPublicInputsTarget<2>,
    pub pt_ml2: ProofWithPublicInputsTarget<2>,
    pub pt_fp12m: ProofWithPublicInputsTarget<2>,
    pub pt_fe: ProofWithPublicInputsTarget<2>,
}

impl ReadTargets for BlsCircuitTargets {
    fn read_targets(
        data: &mut plonky2::util::serialization::Buffer,
    ) -> plonky2::util::serialization::IoResult<Self>
    where
        Self: Sized,
    {
        let pubkey = data.read_target_array::<48>()?;
        let sig = data.read_target_array::<96>()?;
        let msg = data.read_target_array::<32>()?;
        let is_valid_signature = data.read_target()?;

        let pt_pp1 = data.read_target_proof_with_public_inputs()?;
        let pt_pp2 = data.read_target_proof_with_public_inputs()?;
        let pt_ml1 = data.read_target_proof_with_public_inputs()?;
        let pt_ml2 = data.read_target_proof_with_public_inputs()?;
        let pt_fp12m = data.read_target_proof_with_public_inputs()?;
        let pt_fe = data.read_target_proof_with_public_inputs()?;

        Ok(BlsCircuitTargets {
            pubkey,
            sig,
            msg,
            is_valid_signature,
            pt_pp1,
            pt_pp2,
            pt_ml1,
            pt_ml2,
            pt_fp12m,
            pt_fe,
        })
    }
}

impl WriteTargets for BlsCircuitTargets {
    fn write_targets(&self) -> plonky2::util::serialization::IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_array(&self.pubkey)?;
        data.write_target_array(&self.sig)?;
        data.write_target_array(&self.msg)?;
        data.write_target(self.is_valid_signature)?;

        data.write_target_proof_with_public_inputs(&self.pt_pp1)?;
        data.write_target_proof_with_public_inputs(&self.pt_pp2)?;
        data.write_target_proof_with_public_inputs(&self.pt_ml1)?;
        data.write_target_proof_with_public_inputs(&self.pt_ml2)?;
        data.write_target_proof_with_public_inputs(&self.pt_fp12m)?;
        data.write_target_proof_with_public_inputs(&self.pt_fe)?;

        Ok(data)
    }
}

pub fn build_bls12_381_circuit(
    pairing_precomp: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    miller_loop_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    fp12_mul_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    fe_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> (
    BlsCircuitTargets,
    CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::new(config);

    let targets = bls12_381(
        &mut builder,
        &pairing_precomp,
        &miller_loop_circuit_data,
        &fp12_mul_circuit_data,
        &fe_circuit_data,
    );

    let circuit_data = builder.build();

    (targets, circuit_data)
}

type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<2>>::F;
const D: usize = 2;

pub fn bls12_381(
    builder: &mut CircuitBuilder<F, D>,
    pairing_precomp: &CircuitData<F, C, D>,
    miller_loop_circuit_data: &CircuitData<F, C, D>,
    fp12_mul_circuit_data: &CircuitData<F, C, D>,
    fe_circuit_data: &CircuitData<F, C, D>,
) -> BlsCircuitTargets {
    let pubkey = builder.add_virtual_target_arr::<48>();

    let sig_targets = builder.add_virtual_target_arr::<96>();

    let msg_targets = builder.add_virtual_target_arr::<32>();

    let pp_verifier_data = builder.constant_verifier_data(&pairing_precomp.verifier_only);
    let ml_verifier_data = builder.constant_verifier_data(&miller_loop_circuit_data.verifier_only);
    let fp12m_verifier_data = builder.constant_verifier_data(&fp12_mul_circuit_data.verifier_only);
    let fe_verifier_data = builder.constant_verifier_data(&fe_circuit_data.verifier_only);

    let pt_pp1 = builder.add_virtual_proof_with_pis(&pairing_precomp.common);
    builder.verify_proof::<C>(&pt_pp1, &pp_verifier_data, &pairing_precomp.common);
    let pt_pp2 = builder.add_virtual_proof_with_pis(&pairing_precomp.common);
    builder.verify_proof::<C>(&pt_pp2, &pp_verifier_data, &pairing_precomp.common);

    let pt_ml1 = builder.add_virtual_proof_with_pis(&miller_loop_circuit_data.common);
    builder.verify_proof::<C>(&pt_ml1, &ml_verifier_data, &miller_loop_circuit_data.common);
    let pt_ml2 = builder.add_virtual_proof_with_pis(&miller_loop_circuit_data.common);
    builder.verify_proof::<C>(&pt_ml2, &ml_verifier_data, &miller_loop_circuit_data.common);

    let pt_fp12m = builder.add_virtual_proof_with_pis(&fp12_mul_circuit_data.common);
    builder.verify_proof::<C>(
        &pt_fp12m,
        &fp12m_verifier_data,
        &fp12_mul_circuit_data.common,
    );

    let pt_fe = builder.add_virtual_proof_with_pis(&fe_circuit_data.common);
    builder.verify_proof::<C>(&pt_fe, &fe_verifier_data, &fe_circuit_data.common);

    let hm = hash_to_curve(builder, &msg_targets);

    connect_pairing_precomp_with_g2(builder, &pt_pp1, &hm);

    connect_pairing_precomp_with_miller_loop_g2(builder, &pt_pp1, &pt_ml1);

    let pubkey_g1 = get_g1_from_miller_loop(&pt_ml1);
    pk_point_check(builder, &pubkey_g1, &pubkey);

    let signature_g2 = get_g2_point_from_pairing_precomp(builder, &pt_pp2);
    signature_point_check(builder, &signature_g2, &sig_targets);

    connect_pairing_precomp_with_miller_loop_g2(builder, &pt_pp2, &pt_ml2);

    let neg_generator = get_neg_generator(builder);

    connect_miller_loop_with_g1(builder, &neg_generator, &pt_ml2);

    connect_miller_loop_with_fp12_mull(builder, &pt_ml1, &pt_ml2, &pt_fp12m);

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

    // TODO: maybe transfer to bits
    builder.register_public_inputs(&pubkey);
    builder.register_public_inputs(&sig_targets);
    builder.register_public_inputs(&msg_targets);
    builder.register_public_input(is_valid_signature.target);

    BlsCircuitTargets {
        pubkey,
        sig: sig_targets,
        msg: msg_targets,
        is_valid_signature: is_valid_signature.target,
        pt_pp1,
        pt_pp2,
        pt_ml1,
        pt_ml2,
        pt_fp12m,
        pt_fe,
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

fn connect_miller_loop_with_fp12_mull<F: RichField + Extendable<D>, const D: usize>(
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
