use crate::verification::{
    fields::{fp2::Fp2Target, fp::FpTarget},
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
    fn test_hash_to_field_with_msg_123() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            Variable::constant(&mut builder, GoldilocksField(1)),
            Variable::constant(&mut builder, GoldilocksField(2)),
            Variable::constant(&mut builder, GoldilocksField(3)),
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
            BigUint::from_str("606507797311581020112643599031551452644938743645582126724317736403679037685773463889163987655497815884851856762791").unwrap(), 
            BigUint::from_str("3767784791373124759425154252200245266204621939521655132617534948439831126561710118314220667388823309383935776977278").unwrap(), 
            BigUint::from_str("1337261004133155697418998689433745236592976795852159519048998602808295714132539646470083403749308443707762031181885").unwrap(), 
            BigUint::from_str("1166255192842119860213722600682191947347335882189229693319040739546145549855994769170846640327180479164519473877904").unwrap()
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
