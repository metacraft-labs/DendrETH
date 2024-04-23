use crate::verification::{
    fields::plonky2::{fp2_plonky2::Fp2Target, fp_plonky2::FpTarget},
    native::modulus,
    signature::hashing_helpers::SHA256_DIGEST_SIZE,
};
use itertools::Itertools;
use plonky2::{field::types::Field, iop::target::BoolTarget};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{BigUintTarget, CircuitBuilderBiguint},
            u32::gadgets::arithmetic_u32::U32Target,
        },
        vars::{ByteVariable, Bytes32Variable, CircuitVariable, Variable},
    },
};
use std::iter::Iterator;

const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const DST_LEN: usize = DST.len();
const M: usize = 2;
const L: usize = (381 + 128 + 7) / 8; // bls12-381 prime bits - 381, target secutity bits - 128

pub fn i2osp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    value: usize,
    length: usize,
) -> Vec<ByteVariable> {
    if value >= 1 << (8 * length) {
        assert!(false);
    }
    let value = builder.api.constant(L::Field::from_canonical_usize(value));
    let _0xff = builder.api.constant(L::Field::from_canonical_u8(0xff));

    let mut value = ByteVariable::from_target(builder, value);
    let _0xff = ByteVariable::from_target(builder, _0xff);
    let mut res: Vec<ByteVariable> = Vec::with_capacity(length);
    for _ in 0..res.len() {
        res.push(ByteVariable::constant(builder, 0));
    }
    for i in (0..length - 1).rev() {
        println!("i2osp before fail");
        res[i] = builder.and(value, _0xff);
        value = builder.shr(value, 8);
    }

    res
}

pub fn strxor<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &[ByteVariable],
    b: &[ByteVariable],
) -> Vec<ByteVariable> {
    let mut res: Vec<ByteVariable> = Vec::with_capacity(a.len());
    for _ in 0..res.len() {
        res.push(ByteVariable::init_unsafe(builder));
    }
    for i in 0..a.len() {
        res[i] = builder.xor(a[i], b[i]);
    }
    res
}

pub fn concatenate_bytes(bytes: &[&[ByteVariable]]) -> Vec<ByteVariable> {
    let total_length: usize = bytes.iter().map(|byte| byte.len()).sum();
    let mut result = Vec::with_capacity(total_length);
    for byte in bytes {
        result.extend_from_slice(byte);
    }
    result
}

pub fn expand_message_xmd<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    msg: &[ByteVariable],
    dst: &[ByteVariable],
    len_in_bytes: usize,
) -> Vec<ByteVariable> {
    let b_in_bytes = SHA256_DIGEST_SIZE;
    let r_in_bytes = b_in_bytes * 2;
    let ell = (len_in_bytes + b_in_bytes as usize - 1) / b_in_bytes as usize;
    assert!(ell <= 255, "Invalid xmd length");

    let dst_len_octet_stream = i2osp(builder, dst.len(), 1);
    let dst_prime = concatenate_bytes(&[dst, &dst_len_octet_stream]);
    println!("expand_message_xmd |1|");
    println!("r_in_bytes is: {:?}", r_in_bytes);
    let z_pad = i2osp(builder, 0, r_in_bytes as usize);
    let l_i_b_str = i2osp(builder, len_in_bytes, 2);
    let mut b: Vec<Bytes32Variable> = Vec::with_capacity(ell);
    for _ in 0..b.len() {
        b.push(Bytes32Variable::init_unsafe(builder));
    }
    let temp = i2osp(builder, 0, 1);
    let b_0 = builder.curta_sha256(&concatenate_bytes(&[
        &z_pad,
        msg,
        &l_i_b_str,
        &temp,
        &dst_prime.as_slice(),
    ]));
    let temp = i2osp(builder, 1, 1);
    b[0] = builder.curta_sha256(&concatenate_bytes(&[
        &b_0.as_bytes(),
        &temp,
        &dst_prime.as_slice(),
    ]));

    for i in 1..=ell {
        let b_0_xor_bi_m1 = strxor(builder, &b_0.as_bytes(), &b[i - 1].as_bytes());
        let i_1_2osp = i2osp(builder, (i + 1).into(), 1);
        let args = [&b_0_xor_bi_m1, i_1_2osp.as_slice(), &dst_prime];
        b[i] = builder.curta_sha256(&concatenate_bytes(&args[..]));
    }

    let mut r_b: Vec<ByteVariable> = Vec::with_capacity(b.len() * 32);
    for i in 0..b.len() {
        for j in 0..32 {
            r_b.push(b[i].as_bytes()[j]);
        }
    }
    let pseudo_random_bytes = concatenate_bytes(&[&r_b]);
    pseudo_random_bytes[0..len_in_bytes].to_vec()
}

pub fn hash_to_field<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    msg: &[Variable],
    count: usize,
) -> Vec<Fp2Target> {
    let dst_bytes = DST.as_bytes();
    let len_in_bytes = count * M * L;

    let modulus = builder.api.constant_biguint(&modulus());

    let dst = dst_bytes
        .iter()
        .map(|b| builder.constant(L::Field::from_canonical_u32(*b as u32)))
        .collect::<Vec<Variable>>();

    let dst = dst
        .iter()
        .map(|b| ByteVariable::from_variable(builder, *b))
        .collect::<Vec<ByteVariable>>();
    let msg = msg
        .to_vec()
        .iter()
        .map(|f| ByteVariable::from_variable(builder, *f))
        .collect_vec();
    let mut pseudo_random_bytes = expand_message_xmd(builder, &msg, &dst, len_in_bytes);
    pseudo_random_bytes
        .iter_mut()
        .for_each(|big| big.0.reverse());
    let mut u: Vec<Fp2Target> = Vec::with_capacity(count);
    for i in 0..count {
        let mut e: Vec<FpTarget> = Vec::with_capacity(M);
        for j in 0..M {
            let elm_offset = L * (j + i * M);
            let tv = convert_bytesvariable_to_biguint(
                builder,
                &pseudo_random_bytes[elm_offset..elm_offset + L],
            );
            let point = builder.api.rem_biguint(&tv, &modulus);
            e.push(point);
        }
        u.push(e.try_into().unwrap());
    }

    u
}

pub fn convert_bytesvariable_to_biguint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bytes: &[ByteVariable],
) -> BigUintTarget {
    assert!(bytes.len() % 32 == 0);
    let mut u32_targets = Vec::new();

    let mut le_bits: Vec<BoolTarget> = Vec::new();
    for i in 0..bytes.len() {
        le_bits.extend_from_slice(&bytes[i].as_le_bits().map(|x| x.into()));
    }

    for u32_chunk in le_bits.chunks(32) {
        u32_targets.push(U32Target::from_target_unsafe(
            builder.api.le_sum(u32_chunk.iter()),
        ));
    }

    BigUintTarget { limbs: u32_targets }
}

// pub fn _hash_to_field<L: PlonkParameters<D>, const D: usize>(
//     builder: &mut CircuitBuilder<L, D>,
//     msg: &[Variable],
//     count: usize,
// ) -> Vec<Fp2Target> {
//     let dst_bytes = DST.as_bytes();
//     let len_in_bytes = count * M * L;

//     let modulus = builder.api.constant_biguint(&modulus());

//     let dst = dst_bytes
//         .iter()
//         .map(|b| {
//             ByteVariable::from_target(
//                 builder,
//                 builder
//                     .api
//                     .constant(L::Field::from_canonical_u32(*b as u32)),
//             )
//         })
//         .collect::<Vec<ByteVariable>>();
//     let msg = msg
//         .to_vec()
//         .iter()
//         .map(|f| ByteVariable::from_variable(builder, *f))
//         .collect_vec();
//     let mut pseudo_random_bytes = expand_message_xmd(builder, &msg, &dst, len_in_bytes);
//     pseudo_random_bytes
//         .iter_mut()
//         .for_each(|big| big.0.reverse());
//     let mut u: Vec<Fp2Target> = Vec::with_capacity(count);
//     for i in 0..count {
//         let mut e: Vec<FpTarget> = Vec::with_capacity(M);
//         for j in 0..M {
//             let elm_offset = (L * (j + i * M)) / 32;
//             let mut non_reduced_limbs = vec![];
//             for k in (0..L / 32).rev() {
//                 non_reduced_limbs.append(&mut pseudo_random_bytes[elm_offset + k].0);
//             }
//             let non_reduced_point = BigUintTarget {
//                 limbs: non_reduced_limbs,
//             };
//             let point = builder.api.rem_biguint(&non_reduced_point, &modulus);
//             e.push(point);
//         }
//         u.push(e.try_into().unwrap());
//     }

//     u
// }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigUint;
    use plonky2::{
        field::types::Field,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use plonky2x::{
        backend::circuit::DefaultParameters,
        frontend::{
            builder::{CircuitBuilder, DefaultBuilder},
            uint::num::biguint::WitnessBigUint,
            vars::Variable,
        },
    };

    use crate::verification::signature::hash_to_field::{hash_to_field, M};

    #[test]
    fn test_hash_to_field_circuit() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        let config = CircuitConfig::standard_recursion_config();
        // let mut builder = CircuitBuilder::<F, D>::new(config);
        let mut builder = DefaultBuilder::new();
        let msg = vec![0; 0];
        let points = vec![
            [
                BigUint::from_str(
                    "29049427705470064014372021539200946731799999421508007424058975406727614446045474101630850618806446883308850416212"
                ).unwrap(),
                BigUint::from_str(
                    "1902536696277558307181953186589646430378426314321017542292852776971493752529393071590138748612350933458183942594017"
                ).unwrap(),
            ],
            [
                BigUint::from_str(
                    "1469261503385240180838932949518429345203566614064503355039321556894749047984560599095216903263030533722651807245292"
                ).unwrap(),
                BigUint::from_str(
                    "572729459443939985969475830277770585760085104819073756927946494897811696192971610777692627017094870085003613417370"
                ).unwrap(),
            ]
        ];
        let msg_target = builder.api.add_virtual_targets(msg.len());
        let msg_target_var = msg_target
            .iter()
            .map(|t| Variable(*t))
            .collect::<Vec<Variable>>();
        let point_targets = hash_to_field(&mut builder, &msg_target_var, 2);

        builder.api.print_gate_counts(0);
        let data = builder.api.build::<C>();

        let mut pw = PartialWitness::<F>::new();
        let msg_f = msg
            .iter()
            .map(|m| F::from_canonical_u32(*m))
            .collect::<Vec<F>>();
        pw.set_target_arr(&msg_target, &msg_f);
        for i in 0..point_targets.len() {
            for j in 0..M {
                pw.set_biguint_target(&point_targets[i][j], &points[i][j]);
            }
        }

        let proof = data.prove(pw).expect("failed to prove");
        data.verify(proof).expect("failed to verify");
    }

    // #[test]
    // fn test_hash_to_field_circuit() {
    //     type L = DefaultParameters;
    //     const D: usize = 2;

    //     // Define circuit
    //     let mut builder = DefaultBuilder::new();
    //     let msg = vec![0; 0];
    //     let points = vec![
    //         [
    //             BigUint::from_str(
    //                 "29049427705470064014372021539200946731799999421508007424058975406727614446045474101630850618806446883308850416212"
    //             ).unwrap(),
    //             BigUint::from_str(
    //                 "1902536696277558307181953186589646430378426314321017542292852776971493752529393071590138748612350933458183942594017"
    //             ).unwrap(),
    //         ],
    //         [
    //             BigUint::from_str(
    //                 "1469261503385240180838932949518429345203566614064503355039321556894749047984560599095216903263030533722651807245292"
    //             ).unwrap(),
    //             BigUint::from_str(
    //                 "572729459443939985969475830277770585760085104819073756927946494897811696192971610777692627017094870085003613417370"
    //             ).unwrap(),
    //         ]
    //     ];
    //     let msg_target = builder.api.add_virtual_targets(msg.len());
    //     let msg_target_var = msg_target
    //         .iter()
    //         .map(|t| Variable(*t))
    //         .collect::<Vec<Variable>>();
    //     let point_targets = hash_to_field(&mut builder, &msg_target_var, 2);

    //     builder.build();

    //     let msg_f = msg
    //         .iter()
    //         .map(|m| F::from_canonical_u32(*m))
    //         .collect::<Vec<F>>();
    //     pw.set_target_arr(&msg_target, &msg_f);
    //     for i in 0..point_targets.len() {
    //         for j in 0..M {
    //             pw.set_biguint_target(&point_targets[i][j], &points[i][j]);
    //         }
    //     }

    //     let a = builder.read::<Variable>();
    //     let b = builder.read::<Variable>();
    //     let c = builder.add(a, b);
    //     builder.write(c);

    //     // Build circuit
    //     let circuit = builder.build();

    //     let mut input = CIRCUIT.input();

    //     input.write::<CheckpointVariable>(source.clone());

    //     let (proof, output) = CIRCUIT.prove(&input);
    //     CIRCUIT.verify(&proof, &input, &output);

    //     let proof = data.prove(pw).expect("failed to prove");
    //     data.verify(proof).expect("failed to verify");
    // }
}

// #[test]
// fn test_simple_circuit_with_field_io() {
//     utils::setup_logger();
//     // Define your circuit.
//     let mut builder = DefaultBuilder::new();
//     let a = builder.read::<Variable>();
//     let b = builder.read::<Variable>();
//     let c = builder.add(a, b);
//     builder.write(c);

//     // Build your circuit.
//     let circuit = builder.build();

//     // Write to the circuit input.
//     let mut input = circuit.input();
//     input.write::<Variable>(GoldilocksField::TWO);
//     input.write::<Variable>(GoldilocksField::TWO);

//     // Generate a proof.
//     let (proof, mut output) = circuit.prove(&input);

//     // Verify proof.
//     circuit.verify(&proof, &input, &output);

//     // Read output.
//     let sum = output.read::<Variable>();
//     debug!("{}", sum.0);
// }
