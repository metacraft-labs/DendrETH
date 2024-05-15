use crate::verification::{
    fields::{fp::FpTarget, fp2::Fp2Target},
    utils::native_bls::modulus,
};
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
        vars::{ByteVariable, Bytes32Variable, BytesVariable, CircuitVariable},
    },
};
use std::iter::Iterator;

const SHA256_DIGEST_SIZE: u8 = 32;
const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const DST_LEN: usize = DST.len();
const M: usize = 2;
const L: usize = (381 + 128 + 7) / 8;

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
    msg: &[ByteVariable],
    count: usize,
) -> Vec<Fp2Target> {
    let dst_bytes = DST.as_bytes();
    let len_in_bytes = count * M * L;

    let modulus = builder.api.constant_biguint(&modulus());

    let dst = dst_bytes
        .iter()
        .map(|b| {
            let b_v = builder.constant(L::Field::from_canonical_u8(*b));
            ByteVariable::from_variable(builder, b_v)
        })
        .collect::<Vec<ByteVariable>>();

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

    use super::i2osp;
    use crate::verification::aggregation::hash_to_field::{
        expand_message_xmd, hash_to_field, string_to_bytes_target, strxor, DST,
    };
    use itertools::Itertools;
    use num_bigint::BigUint;
    use plonky2::field::{goldilocks_field::GoldilocksField, types::Field};
    use plonky2x::{
        backend::circuit::DefaultParameters,
        frontend::{
            builder::DefaultBuilder,
            uint::num::biguint::BigUintTarget,
            vars::{ByteVariable, BytesVariable, Variable},
        },
    };

    const D: usize = 2;

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

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        let res = [output.read::<ByteVariable>(), output.read::<ByteVariable>()];
        assert_eq!(res[0], 1);
        assert_eq!(res[1], 2);
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

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        let res = [output.read::<ByteVariable>(), output.read::<ByteVariable>()];
        assert_eq!(res[0], 1);
        assert_eq!(res[1], 50);
    }

    #[test]
    fn test_expand_message_xmd() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![1, 2, 3]
            .iter()
            .map(|b| {
                let b_v = builder.constant(GoldilocksField::from_canonical_u8(*b));
                ByteVariable::from_variable(&mut builder, b_v)
            })
            .collect::<Vec<ByteVariable>>();
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
    }

    #[test]
    fn test_hash_to_field() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            103, 140, 163, 210, 238, 252, 75, 8, 227, 27, 60, 229, 125, 150, 241, 222, 217, 156,
            178, 17, 14, 199, 15, 172, 94, 179, 249, 0, 197, 206, 104, 200, 165, 253, 55, 147, 171,
            191, 118, 189, 133, 138, 2, 22, 237, 6, 62, 10, 68, 105, 208, 102, 66, 70, 170, 114,
            194, 80, 215, 5, 63, 95, 202, 1, 99, 153, 67, 115, 7, 122, 235, 255, 142, 44, 3, 65,
            190, 166, 218, 72, 230, 196, 24, 88, 146, 193, 211, 90, 37, 173, 71, 152, 21, 226, 89,
            79, 239, 81, 149, 135, 188, 51, 52, 116, 26, 30, 126, 31, 35, 240, 201, 101, 33, 61,
            220, 192, 86, 47, 214, 243, 224, 136, 50, 56, 42, 233, 148, 244, 203, 198, 195, 120,
            36, 221, 181, 53, 160, 58, 167, 131, 216, 183, 83, 232, 151, 87, 46, 54, 128, 123, 231,
            212, 130, 19, 28, 96, 108, 111, 137, 154, 40, 184, 74, 69, 100, 64, 177, 98, 248, 32,
            12, 97, 49, 187, 39, 159, 168, 247, 29, 246, 209, 110, 77, 73, 20, 23, 174, 143, 93,
            92, 162, 48, 134, 119, 213, 139, 234, 205, 91, 113, 204, 121, 57, 4, 41, 180, 144, 76,
            107, 59, 176, 43, 11, 127, 34, 38, 164, 9, 141, 78, 245, 175, 145, 112, 129, 109, 18,
            250, 85, 16, 124, 182, 242, 158, 84, 219, 13, 207, 186, 82, 157, 132, 225, 236, 45,
            185, 228, 161, 169, 106, 25, 155, 251, 254, 223,
        ]
        .iter()
        .map(|b| {
            let b_v = builder.constant(GoldilocksField::from_canonical_u8(*b));
            ByteVariable::from_variable(&mut builder, b_v)
        })
        .collect::<Vec<ByteVariable>>();
        let hash_to_field_res: Vec<[BigUintTarget; 2]> = hash_to_field(&mut builder, &msg, 2);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..hash_to_field_res.len() {
            for j in 0..hash_to_field_res[i].len() {
                for k in 0..hash_to_field_res[i][j].limbs.len() {
                    builder.write(Variable(hash_to_field_res[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..hash_to_field_res.len() {
            for j in 0..hash_to_field_res[i].len() {
                for _ in 0..hash_to_field_res[i][j].limbs.len() {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..((i * 12) + 12)]
                    .iter()
                    .map(|f| f.0 as u32)
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
            BigUint::from_str("2942673459794500824580128114941241077633586577641169850693178278930447424685332677826186374811489148782362443284608").unwrap(), 
            BigUint::from_str("961863142708046042273452691523472524074450767124819253154800002018881071828353246847707036179733382702893758998301").unwrap(), 
            BigUint::from_str("1730253443889188243699347267983827407041125190502469490045674785753813798266321964653512323237347200806418660750026").unwrap(), 
            BigUint::from_str("373669168086355933912269929736599922994165593229668523008784932595414673068627276883453384670961970510484970528923").unwrap()
        ];

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }
}
