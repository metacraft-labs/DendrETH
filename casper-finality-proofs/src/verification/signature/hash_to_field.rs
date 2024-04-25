use crate::verification::{
    fields::plonky2::{fp2_plonky2::Fp2Target, fp_plonky2::FpTarget},
    native::modulus,
    signature::hashing_helpers::SHA256_DIGEST_SIZE,
};
use itertools::Itertools;
use num_bigint::BigUint;
use plonky2::field::types::Field;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{BigUintTarget, CircuitBuilderBiguint},
            u32::gadgets::arithmetic_u32::U32Target,
        },
        vars::{ByteVariable, Bytes32Variable, BytesVariable, CircuitVariable, Variable},
    },
};
use std::iter::Iterator;

const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const DST_LEN: usize = DST.len();
const M: usize = 2;
const L: usize = (381 + 128 + 7) / 8; // bls12-381 prime bits - 381, target secutity bits - 128

pub fn i2osp<L: PlonkParameters<D>, const D: usize, const LENGHT: usize>(
    builder: &mut CircuitBuilder<L, D>,
    value: usize,
) -> BytesVariable<LENGHT> {
    if value >= 1 << (8 * LENGHT) {
        assert!(false);
    }

    let mut value = value;

    let mut res_u8 = [0; LENGHT];

    for i in (0..LENGHT).rev() {
        res_u8[i] = (value as u8) & 0xff;
        value = value >> 8;
    }

    builder.constant(res_u8)
}

pub fn strxor<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &[ByteVariable],
    b: &[ByteVariable],
) -> Vec<ByteVariable> {
    let mut res: Vec<ByteVariable> = Vec::with_capacity(a.len());
    res.resize(a.len(), ByteVariable::init_unsafe(builder));

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
    const B_IN_BYTES: u8 = SHA256_DIGEST_SIZE;
    const R_IN_BYTES: u8 = B_IN_BYTES * 2;
    let ell = (len_in_bytes + B_IN_BYTES as usize - 1) / B_IN_BYTES as usize;
    assert!(ell <= 255, "Invalid xmd length");

    let dst_len_octet_stream = i2osp::<L, D, 1>(builder, DST_LEN);
    let dst_prime = concatenate_bytes(&[dst, &dst_len_octet_stream.0]);
    let z_pad = i2osp::<L, D, { R_IN_BYTES as usize }>(builder, 0);
    let l_i_b_str = i2osp::<L, D, 2>(builder, len_in_bytes);
    let mut b: Vec<Bytes32Variable> = Vec::with_capacity(ell);
    b.resize(ell + 1, Bytes32Variable::init_unsafe(builder));
    let temp = i2osp::<L, D, 1>(builder, 0);
    let b_0 = builder.curta_sha256(&concatenate_bytes(&[
        &z_pad.0,
        msg,
        &l_i_b_str.0,
        &temp.0,
        &dst_prime.as_slice(),
    ]));
    let temp = i2osp::<L, D, 1>(builder, 1);
    b[0] = builder.curta_sha256(&concatenate_bytes(&[
        &b_0.as_bytes(),
        &temp.0,
        &dst_prime.as_slice(),
    ]));

    for i in 1..=ell {
        let b_0_xor_bi_m1 = strxor(builder, &b_0.as_bytes(), &b[i - 1].as_bytes());
        let i_1_2osp = i2osp::<L, D, 1>(builder, (i + 1).into());
        let args = [&b_0_xor_bi_m1, i_1_2osp.0.as_slice(), &dst_prime];
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
    let pseudo_random_bytes = expand_message_xmd(builder, &msg, &dst, len_in_bytes);
    let mut u: Vec<Fp2Target> = Vec::with_capacity(count);
    for i in 0..count {
        let mut e: Vec<FpTarget> = Vec::with_capacity(M);
        for j in 0..M {
            let elm_offset = L * (j + i * M);
            let tv =
                octet_stream_to_integer(builder, &pseudo_random_bytes[elm_offset..elm_offset + L]);
            let point = builder.api.rem_biguint(&tv, &modulus);
            e.push(point);
        }
        u.push(e.try_into().unwrap());
    }

    u
}

pub fn octet_stream_to_integer<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bytes: &[ByteVariable],
) -> BigUintTarget {
    let mut result = builder.api.zero_biguint();
    let _256 = builder.api.constant_biguint(&BigUint::from(256u64));

    for i in 0..bytes.len() {
        result = builder.api.mul_biguint(&result, &_256);
        let current_byte = bytes[i].to_variable(builder);
        let current_byte_biguint = BigUintTarget {
            limbs: vec![U32Target::from_target_unsafe(current_byte.0)],
        };
        result = builder.api.add_biguint(&result, &current_byte_biguint);
    }

    result
}

pub fn string_to_bytes_target<L: PlonkParameters<D>, const D: usize, const LENGHT: usize>(
    builder: &mut CircuitBuilder<L, D>,
    s: &str,
) -> BytesVariable<LENGHT> {
    let b = string_to_bytes_native(s);
    let mut bytes = [ByteVariable::constant(builder, 0); LENGHT];

    for i in 0..LENGHT {
        let curr_u8 = builder.api.constant(L::Field::from_canonical_u8(b[i]));
        bytes[i] = ByteVariable::from_target(builder, curr_u8);
    }

    BytesVariable(bytes)
}

fn string_to_bytes_native(s: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(s.len());
    for c in s.chars() {
        bytes.push(c as u8);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigUint;
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
    };
    use plonky2x::{
        backend::circuit::DefaultParameters,
        frontend::{
            builder::DefaultBuilder,
            uint::num::biguint::{BigUintTarget, WitnessBigUint},
            vars::{ByteVariable, BytesVariable, Variable},
        },
    };

    use crate::verification::signature::hash_to_field::{
        expand_message_xmd, hash_to_field, octet_stream_to_integer, string_to_bytes_target, strxor,
        DST, M,
    };

    use super::i2osp;

    const D: usize = 2;

    #[test]
    fn test_hash_to_field_circuit() {
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

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

    #[test]
    fn test_i2osp() {
        let mut builder = DefaultBuilder::new();
        let x = i2osp::<DefaultParameters, D, 2>(&mut builder, 258);

        // Define your circuit.
        builder.write(x[0]);
        builder.write(x[1]);

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();
        // input.write::<Variable>(GoldilocksField::from_canonical_u16(258));

        println!("|||||||");
        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        let res = [output.read::<ByteVariable>(), output.read::<ByteVariable>()];
        assert_eq!(res[0], 1);
        assert_eq!(res[1], 2);
        println!("res is: {:?}", res);
    }

    #[test]
    fn test_strxor() {
        let mut builder = DefaultBuilder::new();
        let x = i2osp::<DefaultParameters, D, 2>(&mut builder, 258);
        let y = i2osp::<DefaultParameters, D, 3>(&mut builder, 12444);
        let z = strxor(&mut builder, &x.0, &y.0);

        // Define your circuit.
        builder.write(z[0]);
        builder.write(z[1]);

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();
        // input.write::<Variable>(GoldilocksField::from_canonical_u16(258));

        println!("|||||||");
        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        let res = [output.read::<ByteVariable>(), output.read::<ByteVariable>()];
        assert_eq!(res[0], 1);
        assert_eq!(res[1], 50);
        println!("res is: {:?}", res);
    }

    #[test]
    fn test_expand_message_xmd() {
        let mut builder = DefaultBuilder::new();
        let one = builder.constant(GoldilocksField::from_canonical_u8(1));
        let two = builder.constant(GoldilocksField::from_canonical_u8(2));
        let three = builder.constant(GoldilocksField::from_canonical_u8(3));
        let one = ByteVariable::from_variable(&mut builder, one);
        let two = ByteVariable::from_variable(&mut builder, two);
        let three = ByteVariable::from_variable(&mut builder, three);
        let msg = vec![one, two, three];
        let dst: BytesVariable<43> = string_to_bytes_target(&mut builder, DST);
        let x = expand_message_xmd(&mut builder, &msg, &dst.0, 3);

        // Define your circuit.
        builder.write(x[0]);
        builder.write(x[1]);
        builder.write(x[2]);

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();
        // input.write::<Variable>(GoldilocksField::from_canonical_u16(258));

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        let res = [
            output.read::<ByteVariable>(),
            output.read::<ByteVariable>(),
            output.read::<ByteVariable>(),
        ];
        assert_eq!(res[0], 112);
        assert_eq!(res[1], 160);
        assert_eq!(res[2], 103);
        println!("res is: {:?}", res);
    }

    // #[test]
    // fn test_octet_stream_to_integer() {
    //     const D: usize = 2;
    //     let mut builder = DefaultBuilder::new();
    //     let x = i2osp::<DefaultParameters, D, 3>(&mut builder, 12444);
    //     let y: BigUintTarget = octet_stream_to_integer(&mut builder, &x.0);

    //     // Define your circuit.
    //     builder.write(y);

    //     // Build your circuit.
    //     let circuit = builder.build();

    //     // Write to the circuit input.
    //     let input = circuit.input();
    //     // input.write::<Variable>(GoldilocksField::from_canonical_u16(258));

    //     // Generate a proof.
    //     let (proof, mut output) = circuit.prove(&input);
    //     // Verify proof.
    //     circuit.verify(&proof, &input, &output);

    //     // Read output.
    //     let res = [output.read::<ByteVariable>(), output.read::<ByteVariable>()];
    //     assert_eq!(res[0], 1);
    //     assert_eq!(res[1], 50);
    //     println!("res is: {:?}", res);
    // }
}
