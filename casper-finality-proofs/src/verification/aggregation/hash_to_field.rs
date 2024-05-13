use crate::verification::{
    fields::{fp::FpTarget, fp2::Fp2Target},
    utils::native_bls::modulus,
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
            vars::{ByteVariable, BytesVariable, CircuitVariable, Variable},
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
    fn test_hash_to_field_with_msg_255_bytes() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            Variable::constant(&mut builder, GoldilocksField(103)),
            Variable::constant(&mut builder, GoldilocksField(140)),
            Variable::constant(&mut builder, GoldilocksField(163)),
            Variable::constant(&mut builder, GoldilocksField(210)),
            Variable::constant(&mut builder, GoldilocksField(238)),
            Variable::constant(&mut builder, GoldilocksField(252)),
            Variable::constant(&mut builder, GoldilocksField(75)),
            Variable::constant(&mut builder, GoldilocksField(8)),
            Variable::constant(&mut builder, GoldilocksField(227)),
            Variable::constant(&mut builder, GoldilocksField(27)),
            Variable::constant(&mut builder, GoldilocksField(60)),
            Variable::constant(&mut builder, GoldilocksField(229)),
            Variable::constant(&mut builder, GoldilocksField(125)),
            Variable::constant(&mut builder, GoldilocksField(150)),
            Variable::constant(&mut builder, GoldilocksField(241)),
            Variable::constant(&mut builder, GoldilocksField(222)),
            Variable::constant(&mut builder, GoldilocksField(217)),
            Variable::constant(&mut builder, GoldilocksField(156)),
            Variable::constant(&mut builder, GoldilocksField(178)),
            Variable::constant(&mut builder, GoldilocksField(17)),
            Variable::constant(&mut builder, GoldilocksField(14)),
            Variable::constant(&mut builder, GoldilocksField(199)),
            Variable::constant(&mut builder, GoldilocksField(15)),
            Variable::constant(&mut builder, GoldilocksField(172)),
            Variable::constant(&mut builder, GoldilocksField(94)),
            Variable::constant(&mut builder, GoldilocksField(179)),
            Variable::constant(&mut builder, GoldilocksField(249)),
            Variable::constant(&mut builder, GoldilocksField(0)),
            Variable::constant(&mut builder, GoldilocksField(197)),
            Variable::constant(&mut builder, GoldilocksField(206)),
            Variable::constant(&mut builder, GoldilocksField(104)),
            Variable::constant(&mut builder, GoldilocksField(200)),
            Variable::constant(&mut builder, GoldilocksField(165)),
            Variable::constant(&mut builder, GoldilocksField(253)),
            Variable::constant(&mut builder, GoldilocksField(55)),
            Variable::constant(&mut builder, GoldilocksField(147)),
            Variable::constant(&mut builder, GoldilocksField(171)),
            Variable::constant(&mut builder, GoldilocksField(191)),
            Variable::constant(&mut builder, GoldilocksField(118)),
            Variable::constant(&mut builder, GoldilocksField(189)),
            Variable::constant(&mut builder, GoldilocksField(133)),
            Variable::constant(&mut builder, GoldilocksField(138)),
            Variable::constant(&mut builder, GoldilocksField(2)),
            Variable::constant(&mut builder, GoldilocksField(22)),
            Variable::constant(&mut builder, GoldilocksField(237)),
            Variable::constant(&mut builder, GoldilocksField(6)),
            Variable::constant(&mut builder, GoldilocksField(62)),
            Variable::constant(&mut builder, GoldilocksField(10)),
            Variable::constant(&mut builder, GoldilocksField(68)),
            Variable::constant(&mut builder, GoldilocksField(105)),
            Variable::constant(&mut builder, GoldilocksField(208)),
            Variable::constant(&mut builder, GoldilocksField(102)),
            Variable::constant(&mut builder, GoldilocksField(66)),
            Variable::constant(&mut builder, GoldilocksField(70)),
            Variable::constant(&mut builder, GoldilocksField(170)),
            Variable::constant(&mut builder, GoldilocksField(114)),
            Variable::constant(&mut builder, GoldilocksField(194)),
            Variable::constant(&mut builder, GoldilocksField(80)),
            Variable::constant(&mut builder, GoldilocksField(215)),
            Variable::constant(&mut builder, GoldilocksField(5)),
            Variable::constant(&mut builder, GoldilocksField(63)),
            Variable::constant(&mut builder, GoldilocksField(95)),
            Variable::constant(&mut builder, GoldilocksField(202)),
            Variable::constant(&mut builder, GoldilocksField(1)),
            Variable::constant(&mut builder, GoldilocksField(99)),
            Variable::constant(&mut builder, GoldilocksField(153)),
            Variable::constant(&mut builder, GoldilocksField(67)),
            Variable::constant(&mut builder, GoldilocksField(115)),
            Variable::constant(&mut builder, GoldilocksField(7)),
            Variable::constant(&mut builder, GoldilocksField(122)),
            Variable::constant(&mut builder, GoldilocksField(235)),
            Variable::constant(&mut builder, GoldilocksField(255)),
            Variable::constant(&mut builder, GoldilocksField(142)),
            Variable::constant(&mut builder, GoldilocksField(44)),
            Variable::constant(&mut builder, GoldilocksField(3)),
            Variable::constant(&mut builder, GoldilocksField(65)),
            Variable::constant(&mut builder, GoldilocksField(190)),
            Variable::constant(&mut builder, GoldilocksField(166)),
            Variable::constant(&mut builder, GoldilocksField(218)),
            Variable::constant(&mut builder, GoldilocksField(72)),
            Variable::constant(&mut builder, GoldilocksField(230)),
            Variable::constant(&mut builder, GoldilocksField(196)),
            Variable::constant(&mut builder, GoldilocksField(24)),
            Variable::constant(&mut builder, GoldilocksField(88)),
            Variable::constant(&mut builder, GoldilocksField(146)),
            Variable::constant(&mut builder, GoldilocksField(193)),
            Variable::constant(&mut builder, GoldilocksField(211)),
            Variable::constant(&mut builder, GoldilocksField(90)),
            Variable::constant(&mut builder, GoldilocksField(37)),
            Variable::constant(&mut builder, GoldilocksField(173)),
            Variable::constant(&mut builder, GoldilocksField(71)),
            Variable::constant(&mut builder, GoldilocksField(152)),
            Variable::constant(&mut builder, GoldilocksField(21)),
            Variable::constant(&mut builder, GoldilocksField(226)),
            Variable::constant(&mut builder, GoldilocksField(89)),
            Variable::constant(&mut builder, GoldilocksField(79)),
            Variable::constant(&mut builder, GoldilocksField(239)),
            Variable::constant(&mut builder, GoldilocksField(81)),
            Variable::constant(&mut builder, GoldilocksField(149)),
            Variable::constant(&mut builder, GoldilocksField(135)),
            Variable::constant(&mut builder, GoldilocksField(188)),
            Variable::constant(&mut builder, GoldilocksField(51)),
            Variable::constant(&mut builder, GoldilocksField(52)),
            Variable::constant(&mut builder, GoldilocksField(116)),
            Variable::constant(&mut builder, GoldilocksField(26)),
            Variable::constant(&mut builder, GoldilocksField(30)),
            Variable::constant(&mut builder, GoldilocksField(126)),
            Variable::constant(&mut builder, GoldilocksField(31)),
            Variable::constant(&mut builder, GoldilocksField(35)),
            Variable::constant(&mut builder, GoldilocksField(240)),
            Variable::constant(&mut builder, GoldilocksField(201)),
            Variable::constant(&mut builder, GoldilocksField(101)),
            Variable::constant(&mut builder, GoldilocksField(33)),
            Variable::constant(&mut builder, GoldilocksField(61)),
            Variable::constant(&mut builder, GoldilocksField(220)),
            Variable::constant(&mut builder, GoldilocksField(192)),
            Variable::constant(&mut builder, GoldilocksField(86)),
            Variable::constant(&mut builder, GoldilocksField(47)),
            Variable::constant(&mut builder, GoldilocksField(214)),
            Variable::constant(&mut builder, GoldilocksField(243)),
            Variable::constant(&mut builder, GoldilocksField(224)),
            Variable::constant(&mut builder, GoldilocksField(136)),
            Variable::constant(&mut builder, GoldilocksField(50)),
            Variable::constant(&mut builder, GoldilocksField(56)),
            Variable::constant(&mut builder, GoldilocksField(42)),
            Variable::constant(&mut builder, GoldilocksField(233)),
            Variable::constant(&mut builder, GoldilocksField(148)),
            Variable::constant(&mut builder, GoldilocksField(244)),
            Variable::constant(&mut builder, GoldilocksField(203)),
            Variable::constant(&mut builder, GoldilocksField(198)),
            Variable::constant(&mut builder, GoldilocksField(195)),
            Variable::constant(&mut builder, GoldilocksField(120)),
            Variable::constant(&mut builder, GoldilocksField(36)),
            Variable::constant(&mut builder, GoldilocksField(221)),
            Variable::constant(&mut builder, GoldilocksField(181)),
            Variable::constant(&mut builder, GoldilocksField(53)),
            Variable::constant(&mut builder, GoldilocksField(160)),
            Variable::constant(&mut builder, GoldilocksField(58)),
            Variable::constant(&mut builder, GoldilocksField(167)),
            Variable::constant(&mut builder, GoldilocksField(131)),
            Variable::constant(&mut builder, GoldilocksField(216)),
            Variable::constant(&mut builder, GoldilocksField(183)),
            Variable::constant(&mut builder, GoldilocksField(83)),
            Variable::constant(&mut builder, GoldilocksField(232)),
            Variable::constant(&mut builder, GoldilocksField(151)),
            Variable::constant(&mut builder, GoldilocksField(87)),
            Variable::constant(&mut builder, GoldilocksField(46)),
            Variable::constant(&mut builder, GoldilocksField(54)),
            Variable::constant(&mut builder, GoldilocksField(128)),
            Variable::constant(&mut builder, GoldilocksField(123)),
            Variable::constant(&mut builder, GoldilocksField(231)),
            Variable::constant(&mut builder, GoldilocksField(212)),
            Variable::constant(&mut builder, GoldilocksField(130)),
            Variable::constant(&mut builder, GoldilocksField(19)),
            Variable::constant(&mut builder, GoldilocksField(28)),
            Variable::constant(&mut builder, GoldilocksField(96)),
            Variable::constant(&mut builder, GoldilocksField(108)),
            Variable::constant(&mut builder, GoldilocksField(111)),
            Variable::constant(&mut builder, GoldilocksField(137)),
            Variable::constant(&mut builder, GoldilocksField(154)),
            Variable::constant(&mut builder, GoldilocksField(40)),
            Variable::constant(&mut builder, GoldilocksField(184)),
            Variable::constant(&mut builder, GoldilocksField(74)),
            Variable::constant(&mut builder, GoldilocksField(69)),
            Variable::constant(&mut builder, GoldilocksField(100)),
            Variable::constant(&mut builder, GoldilocksField(64)),
            Variable::constant(&mut builder, GoldilocksField(177)),
            Variable::constant(&mut builder, GoldilocksField(98)),
            Variable::constant(&mut builder, GoldilocksField(248)),
            Variable::constant(&mut builder, GoldilocksField(32)),
            Variable::constant(&mut builder, GoldilocksField(12)),
            Variable::constant(&mut builder, GoldilocksField(97)),
            Variable::constant(&mut builder, GoldilocksField(49)),
            Variable::constant(&mut builder, GoldilocksField(187)),
            Variable::constant(&mut builder, GoldilocksField(39)),
            Variable::constant(&mut builder, GoldilocksField(159)),
            Variable::constant(&mut builder, GoldilocksField(168)),
            Variable::constant(&mut builder, GoldilocksField(247)),
            Variable::constant(&mut builder, GoldilocksField(29)),
            Variable::constant(&mut builder, GoldilocksField(246)),
            Variable::constant(&mut builder, GoldilocksField(209)),
            Variable::constant(&mut builder, GoldilocksField(110)),
            Variable::constant(&mut builder, GoldilocksField(77)),
            Variable::constant(&mut builder, GoldilocksField(73)),
            Variable::constant(&mut builder, GoldilocksField(20)),
            Variable::constant(&mut builder, GoldilocksField(23)),
            Variable::constant(&mut builder, GoldilocksField(174)),
            Variable::constant(&mut builder, GoldilocksField(143)),
            Variable::constant(&mut builder, GoldilocksField(93)),
            Variable::constant(&mut builder, GoldilocksField(92)),
            Variable::constant(&mut builder, GoldilocksField(162)),
            Variable::constant(&mut builder, GoldilocksField(48)),
            Variable::constant(&mut builder, GoldilocksField(134)),
            Variable::constant(&mut builder, GoldilocksField(119)),
            Variable::constant(&mut builder, GoldilocksField(213)),
            Variable::constant(&mut builder, GoldilocksField(139)),
            Variable::constant(&mut builder, GoldilocksField(234)),
            Variable::constant(&mut builder, GoldilocksField(205)),
            Variable::constant(&mut builder, GoldilocksField(91)),
            Variable::constant(&mut builder, GoldilocksField(113)),
            Variable::constant(&mut builder, GoldilocksField(204)),
            Variable::constant(&mut builder, GoldilocksField(121)),
            Variable::constant(&mut builder, GoldilocksField(57)),
            Variable::constant(&mut builder, GoldilocksField(4)),
            Variable::constant(&mut builder, GoldilocksField(41)),
            Variable::constant(&mut builder, GoldilocksField(180)),
            Variable::constant(&mut builder, GoldilocksField(144)),
            Variable::constant(&mut builder, GoldilocksField(76)),
            Variable::constant(&mut builder, GoldilocksField(107)),
            Variable::constant(&mut builder, GoldilocksField(59)),
            Variable::constant(&mut builder, GoldilocksField(176)),
            Variable::constant(&mut builder, GoldilocksField(43)),
            Variable::constant(&mut builder, GoldilocksField(11)),
            Variable::constant(&mut builder, GoldilocksField(127)),
            Variable::constant(&mut builder, GoldilocksField(34)),
            Variable::constant(&mut builder, GoldilocksField(38)),
            Variable::constant(&mut builder, GoldilocksField(164)),
            Variable::constant(&mut builder, GoldilocksField(9)),
            Variable::constant(&mut builder, GoldilocksField(141)),
            Variable::constant(&mut builder, GoldilocksField(78)),
            Variable::constant(&mut builder, GoldilocksField(245)),
            Variable::constant(&mut builder, GoldilocksField(175)),
            Variable::constant(&mut builder, GoldilocksField(145)),
            Variable::constant(&mut builder, GoldilocksField(112)),
            Variable::constant(&mut builder, GoldilocksField(129)),
            Variable::constant(&mut builder, GoldilocksField(109)),
            Variable::constant(&mut builder, GoldilocksField(18)),
            Variable::constant(&mut builder, GoldilocksField(250)),
            Variable::constant(&mut builder, GoldilocksField(85)),
            Variable::constant(&mut builder, GoldilocksField(16)),
            Variable::constant(&mut builder, GoldilocksField(124)),
            Variable::constant(&mut builder, GoldilocksField(182)),
            Variable::constant(&mut builder, GoldilocksField(242)),
            Variable::constant(&mut builder, GoldilocksField(158)),
            Variable::constant(&mut builder, GoldilocksField(84)),
            Variable::constant(&mut builder, GoldilocksField(219)),
            Variable::constant(&mut builder, GoldilocksField(13)),
            Variable::constant(&mut builder, GoldilocksField(207)),
            Variable::constant(&mut builder, GoldilocksField(186)),
            Variable::constant(&mut builder, GoldilocksField(82)),
            Variable::constant(&mut builder, GoldilocksField(157)),
            Variable::constant(&mut builder, GoldilocksField(132)),
            Variable::constant(&mut builder, GoldilocksField(225)),
            Variable::constant(&mut builder, GoldilocksField(236)),
            Variable::constant(&mut builder, GoldilocksField(45)),
            Variable::constant(&mut builder, GoldilocksField(185)),
            Variable::constant(&mut builder, GoldilocksField(228)),
            Variable::constant(&mut builder, GoldilocksField(161)),
            Variable::constant(&mut builder, GoldilocksField(169)),
            Variable::constant(&mut builder, GoldilocksField(106)),
            Variable::constant(&mut builder, GoldilocksField(25)),
            Variable::constant(&mut builder, GoldilocksField(155)),
            Variable::constant(&mut builder, GoldilocksField(251)),
            Variable::constant(&mut builder, GoldilocksField(254)),
            Variable::constant(&mut builder, GoldilocksField(223)),
        ];
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

    #[test]
    fn test_hash_to_field_with_msg_00() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            Variable::constant(&mut builder, GoldilocksField(0)),
            Variable::constant(&mut builder, GoldilocksField(0)),
        ];
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
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| f.0 as u32)
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
            BigUint::from_str("3467206824264915314410328089509568219675264638553717676707032754344263708519856531598030919370345000939272262922340").unwrap(), 
            BigUint::from_str("967261753488201268360197178061348169493533720124286268768722457533390452889003640920658401692376512788045590844589").unwrap(), 
            BigUint::from_str("1694294209433536606759431236849684172181087494531715444756984659355647441866512912746959707332669387366840278465798").unwrap(), 
            BigUint::from_str("2004324217974516925171115353648739595566178169751049984497836828645148247251982057973578533159710504000584560806028").unwrap()
        ];

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }
}
