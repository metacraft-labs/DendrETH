use plonky2::iop::target::BoolTarget;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{BigUintTarget, CircuitBuilderBiguint},
            u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
        },
        vars::{ByteVariable, Variable},
    },
};

use crate::verification::{
    fields::plonky2::{fp2_plonky2::Fp2Target, fp_plonky2::FpTarget},
    native::modulus,
    signature::hashing_helpers::{
        add_virtual_hash_input_target, and_u32, hash_sha256, rsh_u32, xor_u32, SHA256_DIGEST_SIZE,
    },
};

const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const DST_LEN: usize = DST.len();
const M: usize = 2;
const L: usize = (381 + 128 + 7) / 8; // bls12-381 prime bits - 381, target secutity bits - 128

pub fn preprocess1_sha256_input<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input_bytes: &[U32Target],
    hash_len: usize,
) -> BigUintTarget {
    let zero = builder.api.zero();
    let one = builder.api.one();

    let input_bits_len = input_bytes.len() * 8;
    let next_32_multiple = (input_bits_len + 7 + 31) / 32;

    let mut input_bits = input_bytes
        .iter()
        .flat_map(|byte| builder.api.split_le(byte.target, 8))
        .collect::<Vec<BoolTarget>>();
    input_bits.resize(next_32_multiple * 32, BoolTarget::new_unsafe(zero));
    input_bits[input_bits_len + 7] = BoolTarget::new_unsafe(one);

    let mut input_u32s = input_bits
        .chunks(32)
        .map(|bits| {
            let swap_bits = bits.chunks(8).rev().flatten();
            U32Target::from_target_unsafe(builder.api.le_sum(swap_bits))
        })
        .collect::<Vec<U32Target>>();

    input_u32s.resize(hash_len, U32Target::from_target_unsafe(zero));

    let padding_end1 = builder.api.constant_u32((input_bits_len >> 32) as u32);
    let padding_end0 = builder.api.constant_u32(input_bits_len as u32);
    input_u32s[hash_len - 2] = padding_end1;
    input_u32s[hash_len - 1] = padding_end0;

    BigUintTarget { limbs: input_u32s }
}

pub fn preprocess2_sha256_input<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    prev_hash: &BigUintTarget,
    input_bytes: &[U32Target],
    hash_len: usize,
) -> BigUintTarget {
    let zero = builder.api.zero();
    let one = builder.api.one();

    let input_bits_len = input_bytes.len() * 8;
    let next_32_multiple = (input_bits_len + 7 + 31) / 32;

    let mut input_bits = input_bytes
        .iter()
        .flat_map(|byte| builder.api.split_le(byte.target, 8))
        .collect::<Vec<BoolTarget>>();
    input_bits.resize(next_32_multiple * 32, BoolTarget::new_unsafe(zero));
    input_bits[input_bits_len + 7] = BoolTarget::new_unsafe(one);

    let mut tmp_u32s = input_bits
        .chunks(32)
        .map(|bits| {
            let swap_bits = bits.chunks(8).rev().flatten();
            U32Target::from_target_unsafe(builder.api.le_sum(swap_bits))
        })
        .collect::<Vec<U32Target>>();

    let mut input_u32s = Vec::with_capacity(hash_len);
    for i in 0..prev_hash.limbs.len() {
        input_u32s.push(prev_hash.limbs[i]);
    }
    input_u32s.append(&mut tmp_u32s);
    input_u32s.resize(hash_len, U32Target::from_target_unsafe(zero));

    let padding_end1 = builder
        .api
        .constant_u32(((input_bits_len + 256) >> 32) as u32);
    let padding_end0 = builder.api.constant_u32((input_bits_len + 256) as u32);
    input_u32s[hash_len - 2] = padding_end1;
    input_u32s[hash_len - 1] = padding_end0;

    BigUintTarget { limbs: input_u32s }
}

pub fn i2osp(value: U32Target, length: usize) {}

pub fn expand_message_xmd<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    msg: &[ByteVariable],
    dst: &[ByteVariable],
    len_in_bytes: usize,
) -> Vec<ByteVariable> {
    let b_in_bytes = SHA256_DIGEST_SIZE; // SHA256 output length
    let r_in_bytes = b_in_bytes * 2;
    let ell = (len_in_bytes as u8 + b_in_bytes as u8 - 1) / b_in_bytes;
    assert!(ell <= 255, "Invalid xmd length");

    //////////////////////////////////////////////////////////////////////////

    let dst_prime;

    let zero = builder.api.zero();
    let one = builder.api.one();

    let dst_prime = builder.api.add_virtual_u32_targets_unsafe(DST_LEN + 1);
    for i in 0..DST_LEN {
        builder.api.connect_u32(dst[i], dst_prime[i]);
    }
    let io2sp_dst = builder.api.constant_u32(dst.len() as u32);
    builder.api.connect_u32(dst_prime[DST_LEN], io2sp_dst);
    let z_pad = builder.api.add_virtual_u32_targets_unsafe(r_in_bytes);
    for target in z_pad.iter() {
        builder.api.connect(target.target, zero);
    }
    let l_i_b_str = builder.api.add_virtual_u32_targets_unsafe(2);
    let l_i_b_target = builder.api.constant_u32(len_in_bytes as u32);
    let u8_max = builder.api.constant_u32(0xff);
    let low = and_u32(l_i_b_target, u8_max);
    let high = rsh_u32(builder, l_i_b_target, 8);

    builder.api.connect_u32(l_i_b_str[0], high);
    builder.api.connect_u32(l_i_b_str[1], low);

    let input_len = z_pad.len() + msg.len() + l_i_b_str.len() + 1 + dst_prime.len();

    let mut input_bytes: Vec<ByteVariable> = vec![];
    for i in 0..z_pad.len() {
        input_bytes.push(ByteVariable::from_target(builder, z_pad[i].target));
    }
    for i in 0..msg.len() {
        input_bytes.push(ByteVariable::from_target(builder, msg[i].0));
    }
    for i in 0..l_i_b_str.len() {
        input_bytes.push(ByteVariable::from_target(builder, l_i_b_str[i].target));
    }
    input_bytes.push(ByteVariable::from_target(builder, zero));
    for i in 0..dst_prime.len() {
        input_bytes.push(ByteVariable::from_target(builder, dst_prime[i].target));
    }

    let b_0 = builder.sha256(&input_bytes);

    let mut b = vec![];

    let b0_input = add_virtual_hash_input_target(((32 + 1 + 43) * 8 + 511) / 512, 512);
    let mut b0_input_bytes = vec![];
    b0_input_bytes.push(U32Target::from_target_unsafe(one));
    for i in 0..dst_prime.len() {
        b0_input_bytes.push(dst_prime[i]);
    }
    let preprocessed_input =
        preprocess2_sha256_input(builder, &b_0, &b0_input_bytes, b0_input.input.num_limbs());
    builder
        .api
        .connect_biguint(&preprocessed_input, &b0_input.input);
    let b0 = hash_sha256(&b0_input);
    b.push(b0);

    for i in 1..ell {
        let bi_input = builder.add_virtual_hash_input_target(((32 + 1 + 43) * 8 + 511) / 512, 512);
        let mut bi_input_bytes = vec![];
        let i2osp_i = builder.api.constant_u32((i + 1) as u32);
        bi_input_bytes.push(i2osp_i);
        for i in 0..dst_prime.len() {
            bi_input_bytes.push(dst_prime[i]);
        }
        let prev_xor = BigUintTarget {
            limbs: b_0
                .limbs
                .iter()
                .zip(b[i - 1].limbs.iter())
                .map(|(b0, bi)| xor_u32(*b0, *bi))
                .collect::<Vec<U32Target>>(),
        };
        let preprocessed_input = preprocess2_sha256_input(
            builder,
            &prev_xor,
            &bi_input_bytes,
            bi_input.input.num_limbs(),
        );
        builder
            .api
            .connect_biguint(&preprocessed_input, &bi_input.input);
        let bi = hash_sha256(&bi_input);
        b.push(bi);
    }
    b
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
        .map(|b| builder.api.constant_u32(*b as u32))
        .collect::<Vec<U32Target>>();
    let mut pseudo_random_bytes = expand_message_xmd(builder, &msg, &dst, len_in_bytes);
    pseudo_random_bytes
        .iter_mut()
        .for_each(|big| big.limbs.reverse());
    let mut u: Vec<Fp2Target> = Vec::with_capacity(count);
    for i in 0..count {
        let mut e: Vec<FpTarget> = Vec::with_capacity(M);
        for j in 0..M {
            let elm_offset = (L * (j + i * M)) / 32;
            let mut non_reduced_limbs = vec![];
            for k in (0..L / 32).rev() {
                non_reduced_limbs.append(&mut pseudo_random_bytes[elm_offset + k].limbs);
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
