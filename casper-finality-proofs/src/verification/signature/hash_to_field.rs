use crate::verification::{
    fields::plonky2::{fp2_plonky2::Fp2Target, fp_plonky2::FpTarget},
    native::modulus,
    signature::hashing_helpers::SHA256_DIGEST_SIZE,
};
use itertools::Itertools;
use plonky2::field::types::Field;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::biguint::{BigUintTarget, CircuitBuilderBiguint},
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
    if value < 0 || value >= 1 << (8 * length) {
        assert!(false);
    }
    let value = builder.api.constant(L::Field::from_canonical_usize(length));
    let _0xff = builder.api.constant(L::Field::from_canonical_u8(0xff));

    let mut value = ByteVariable::from_target(builder, value);
    let _0xff = ByteVariable::from_target(builder, _0xff);
    let mut res: Vec<ByteVariable> = Vec::with_capacity(length);
    for _ in 0..res.len() {
        res.push(ByteVariable::init_unsafe(builder));
    }
    for i in (0..length - 1).rev() {
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
        .map(|b| {
            ByteVariable::from_target(
                builder,
                builder
                    .api
                    .constant(L::Field::from_canonical_u32(*b as u32)),
            )
        })
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
            let elm_offset = (L * (j + i * M)) / 32;
            let mut non_reduced_limbs = vec![];
            for k in (0..L / 32).rev() {
                let o = pseudo_random_bytes[elm_offset + k].;
                non_reduced_limbs.append(&mut pseudo_random_bytes[elm_offset + k].0);
            }
            let non_reduced_point = BigUintTarget {
                limbs: non_reduced_limbs,
            };
            let point = builder.api.rem_biguint(&non_reduced_point, &modulus);
            e.push(point);
        }
        u.push(e.try_into().unwrap());
    }

    u
}
