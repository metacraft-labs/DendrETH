use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;
use sha2::{Digest, Sha256};

use super::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    hashing::is_valid_merkle_branch::MerkleBranch,
};

pub const ETH_SHA256_BIT_SIZE: usize = 256;
pub const POSEIDON_HASH_SIZE: usize = 4;

pub fn hex_string_from_field_element_bits<F: RichField + Extendable<D>, const D: usize>(
    bits: &[F],
) -> String {
    assert!(bits.len() % 4 == 0);
    let bits = bits
        .iter()
        .map(|element| element.to_canonical_u64() != 0)
        .collect_vec();

    hex::encode(bits_to_bytes(&bits))
}

pub fn biguint_from_limbs_target(limbs: &[Target]) -> BigUintTarget {
    BigUintTarget {
        limbs: limbs.iter().cloned().map(|x| U32Target(x)).collect_vec(),
    }
}

pub fn biguint_from_field_elements<F: RichField + Extendable<D>, const D: usize>(
    limbs: &[F],
) -> BigUint {
    BigUint::from_slice(
        limbs
            .iter()
            .map(|element| element.to_canonical_u64() as u32)
            .collect_vec()
            .as_slice(),
    )
}

pub fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn target_to_le_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    number: Target,
) -> [BoolTarget; 64] {
    builder
        .split_le(number, 64)
        .into_iter()
        .rev()
        .collect_vec()
        .try_into()
        .unwrap()
}

pub fn biguint_is_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &BigUintTarget,
    b: &BigUintTarget,
) -> BoolTarget {
    assert!(a.limbs.len() == b.limbs.len());

    let mut all_equal = builder._true();

    for i in 0..a.limbs.len() {
        let equal = builder.is_equal(a.limbs[i].0, b.limbs[i].0);
        all_equal = builder.and(all_equal, equal);
    }

    all_equal
}

pub fn bool_target_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &[BoolTarget; ETH_SHA256_BIT_SIZE],
    b: &[BoolTarget; ETH_SHA256_BIT_SIZE],
) -> BoolTarget {
    let mut all_equal = builder._true();

    for i in 0..ETH_SHA256_BIT_SIZE {
        let equal = builder.is_equal(a[i].target, b[i].target);
        all_equal = builder.and(all_equal, equal);
    }

    all_equal
}

pub fn create_sha256_merkle_proof<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
) -> MerkleBranch<DEPTH> {
    [(); DEPTH].map(|_| create_bool_target_array(builder))
}

pub fn create_bool_target_array<
    F: RichField + Extendable<D>,
    const D: usize,
    const TARGETS_COUNT: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; TARGETS_COUNT] {
    (0..TARGETS_COUNT)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn bits_to_biguint_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits_target: Vec<BoolTarget>,
) -> BigUintTarget {
    let bit_len = bits_target.len();
    assert_eq!(bit_len % 32, 0);

    let mut u32_targets = Vec::new();
    for i in 0..bit_len / 32 {
        u32_targets.push(U32Target(
            builder.le_sum(bits_target[i * 32..(i + 1) * 32].iter().rev()),
        ));
    }
    u32_targets.reverse();
    BigUintTarget { limbs: u32_targets }
}

pub fn biguint_to_bits_target<F: RichField + Extendable<D>, const D: usize, const B: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &BigUintTarget,
) -> Vec<BoolTarget> {
    let mut res = Vec::new();
    for i in (0..a.num_limbs()).rev() {
        let bit_targets = builder.split_le_base::<B>(a.get_limb(i).0, 32);
        for j in (0..32).rev() {
            res.push(BoolTarget::new_unsafe(bit_targets[j]));
        }
    }

    res
}

pub fn biguint_to_le_bits_target<F: RichField + Extendable<D>, const D: usize, const B: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &BigUintTarget,
) -> Vec<BoolTarget> {
    biguint_to_bits_target::<F, D, B>(builder, a)
        .into_iter()
        .rev()
        .collect_vec()
}
pub fn _right_rotate<const S: usize>(n: [BoolTarget; S], bits: usize) -> [BoolTarget; S] {
    let mut res = [None; S];
    for i in 0..S {
        res[i] = Some(n[((S - bits) + i) % S])
    }
    res.map(|x| x.unwrap())
}

pub fn _shr<F: RichField + Extendable<D>, const D: usize, const S: usize>(
    n: [BoolTarget; S],
    bits: i64,
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; S] {
    let mut res = [None; S];
    for i in 0..S {
        if (i as i64) < bits {
            res[i] = Some(BoolTarget::new_unsafe(builder.constant(F::ZERO)));
        } else {
            res[i] = Some(n[(i as i64 - bits) as usize]);
        }
    }
    res.map(|x| x.unwrap())
}

pub fn uint32_to_bits<F: RichField + Extendable<D>, const D: usize>(
    value: u32,
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; 32] {
    let mut bits = [None; 32];
    for i in 0..32 {
        if value & (1 << (31 - i)) != 0 {
            bits[i] = Some(BoolTarget::new_unsafe(builder.constant(F::ONE)));
        } else {
            bits[i] = Some(BoolTarget::new_unsafe(builder.constant(F::ZERO)));
        }
    }
    bits.map(|x| x.unwrap())
}

fn reverse_endianness(bits: &[BoolTarget]) -> Vec<BoolTarget> {
    bits.chunks(8).rev().flatten().cloned().collect()
}

pub fn ssz_num_to_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    num: &BigUintTarget,
    bit_len: usize,
) -> Vec<BoolTarget> {
    assert!(bit_len <= ETH_SHA256_BIT_SIZE);

    let mut bits = reverse_endianness(&biguint_to_bits_target::<F, D, 2>(builder, num));
    bits.extend((bit_len..ETH_SHA256_BIT_SIZE).map(|_| builder._false()));

    bits
}

pub fn ssz_num_from_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits: &[BoolTarget],
) -> BigUintTarget {
    bits_to_biguint_target(builder, reverse_endianness(bits))
}

pub fn select_biguint<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    b: BoolTarget,
    x: &BigUintTarget,
    y: &BigUintTarget,
) -> BigUintTarget {
    let not_b = builder.not(b);

    let maybe_x = builder.mul_biguint_by_bool(x, b);

    let maybe_y = builder.mul_biguint_by_bool(y, not_b);

    let mut result = builder.add_biguint(&maybe_y, &maybe_x);

    // trim the carry
    result.limbs.pop();

    result
}

pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::new();

    for value in bytes {
        for i in (0..8).rev() {
            let mask = 1 << i;
            bits.push(value & mask != 0);
        }
    }

    bits
}

pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = 0u8;

    for (index, bit) in bits.iter().enumerate() {
        if *bit {
            byte |= 1 << (7 - (index % 8));
        }

        if index % 8 == 7 {
            bytes.push(byte);
            byte = 0;
        }
    }

    if bits.len() % 8 != 0 {
        bytes.push(byte);
    }

    bytes
}

pub trait SetBytesArray<F: Field> {
    fn set_bytes_array(&mut self, targets: &[BoolTarget], values: &[u8]);
}

impl<F: Field> SetBytesArray<F> for PartialWitness<F> {
    fn set_bytes_array(&mut self, targets: &[BoolTarget], values: &[u8]) {
        assert!(targets.len() == values.len() * 8);

        let bool_values = bytes_to_bits(values);

        for i in 0..targets.len() {
            self.set_bool_target(targets[i], bool_values[i]);
        }
    }
}

#[cfg(test)]
mod test_ssz_num_from_bits {
    use anyhow::Result;
    use itertools::Itertools;
    use num::{BigUint, Num};
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use serde::Deserialize;
    use std::{fs, iter::repeat, println};

    use crate::utils::{biguint::CircuitBuilderBiguint, utils::ssz_num_from_bits};

    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    struct Config {
        test_cases: Vec<TestCase>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct TestCase {
        r#type: String,
        valid: bool,
        value: String,
        ssz: Option<String>,
        tags: Vec<String>,
    }

    fn get_test_cases(path: &str) -> Result<Vec<TestCase>> {
        let yaml_str = fs::read_to_string(path).expect("Unable to read config file");
        let config: Config = serde_yaml::from_str(&yaml_str)?;

        Ok(config.test_cases)
    }

    #[test]
    fn test_ssz_num_from_bits() -> Result<()> {
        let bound_test_cases = get_test_cases("../../../vendor/eth2.0-tests/ssz/uint_bounds.yaml")?
            .iter()
            .cloned()
            .filter(|x| x.valid)
            .collect_vec();

        let random_test_cases =
            get_test_cases("../../../vendor/eth2.0-tests/ssz/uint_random.yaml")?
                .iter()
                .cloned()
                .filter(|x| x.valid)
                .collect_vec();

        let test_cases = bound_test_cases
            .iter()
            .chain(random_test_cases.iter())
            .cloned()
            .collect_vec();

        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let grouped_test_cases = test_cases
            .iter()
            .group_by(|x| x.r#type.clone())
            .into_iter()
            .map(|(k, v)| (k, v.cloned().collect_vec()))
            .collect_vec();

        for (type_, test_cases) in grouped_test_cases {
            let num_bits = type_
                .split("uint")
                .last()
                .unwrap()
                .parse::<usize>()
                .unwrap();

            if num_bits % 32 != 0 {
                // For  now lets test only multiples of 32
                continue;
            }

            for test_case in test_cases {
                println!(
                    "Running test case: {}_{}",
                    test_case.r#type, test_case.tags[2]
                );

                let mut pw = PartialWitness::new();

                let mut builder =
                    CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

                let bits = (0..num_bits)
                    .map(|_| builder.add_virtual_bool_target_safe())
                    .collect::<Vec<_>>();

                let target = ssz_num_from_bits(&mut builder, &bits);

                let value = test_case.value.parse::<BigUint>().expect(
                    format!(
                        "Unable to parse value: {}_{}",
                        test_case.r#type, test_case.tags[2]
                    )
                    .as_str(),
                );

                let expected_target = builder.constant_biguint(&value);

                builder.connect_biguint(&target, &expected_target);

                let data = builder.build::<C>();

                let bits_value = BigUint::from_str_radix(&test_case.ssz.unwrap()[2..], 16)
                    .unwrap()
                    .to_str_radix(2)
                    .chars()
                    .map(|x| x == '1')
                    .collect_vec();

                let padding_length = num_bits - bits_value.len();

                let expected_bits = repeat(false)
                    .take(padding_length)
                    .chain(bits_value.iter().cloned())
                    .collect_vec();

                for i in 0..num_bits {
                    pw.set_bool_target(bits[i], expected_bits[i]);
                }

                let proof = data.prove(pw).expect(
                    format!(
                        "Prove failed for {}_{}",
                        test_case.r#type, test_case.tags[2]
                    )
                    .as_str(),
                );

                data.verify(proof).expect(
                    format!(
                        "Prove failed for {}_{}",
                        test_case.r#type, test_case.tags[2]
                    )
                    .as_str(),
                );
            }
        }

        Ok(())
    }
}
