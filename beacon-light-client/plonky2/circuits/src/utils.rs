use num::BigUint;
use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;
use sha2::{Digest, Sha256};

use crate::biguint::{BigUintTarget, CircuitBuilderBiguint};

pub const ETH_SHA256_BIT_SIZE: usize = 256;
pub const POSEIDON_HASH_SIZE: usize = 4;

pub fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn biguint_same_limbs_is_equal<F: RichField + Extendable<D>, const D: usize>(
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

pub fn select_biguint_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    b: BoolTarget,
    x: BigUintTarget,
    y: BigUintTarget,
) -> BigUintTarget {
    let tmp = mul_sub_biguint(builder, b, &y, &y);
    mul_sub_biguint(builder, b, &x, &tmp)
}

fn mul_sub_biguint<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: BoolTarget,
    y: &BigUintTarget,
    z: &BigUintTarget,
) -> BigUintTarget {
    let prod = builder.mul_biguint_by_bool(&y, x);
    builder.sub_biguint(&prod, z)
}

pub fn get_validator_relevance<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    activation_epoch: &BigUintTarget,
    current_epoch: &BigUintTarget,
    withdrawable_epoch: &BigUintTarget,
) -> BoolTarget {
    let current_le_withdrawable_epoch = builder.cmp_biguint(&current_epoch, &withdrawable_epoch);
    let activation_epoch_le_current_epoch = builder.cmp_biguint(&activation_epoch, &current_epoch);

    builder.and(
        current_le_withdrawable_epoch,
        activation_epoch_le_current_epoch,
    )
}

pub fn is_equal_u32<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: U32Target,
    y: U32Target,
) -> BoolTarget {
    builder.is_equal(x.0, y.0)
}

pub fn biguint_is_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &BigUintTarget,
    b: &BigUintTarget,
) -> BoolTarget {
    let mut ret = builder._true();
    let false_t = builder._false().target;

    let min_limbs = a.num_limbs().min(b.num_limbs());
    for i in 0..min_limbs {
        let limb_equal = is_equal_u32(builder, a.get_limb(i), b.get_limb(i));
        ret = BoolTarget::new_unsafe(builder.select(limb_equal, ret.target, false_t));
    }

    let zero_u32 = U32Target(builder.zero());
    for i in min_limbs..a.num_limbs() {
        let is_zero = is_equal_u32(builder, a.get_limb(i), zero_u32);
        ret = BoolTarget::new_unsafe(builder.select(is_zero, ret.target, false_t));
    }
    for i in min_limbs..b.num_limbs() {
        let is_zero = is_equal_u32(builder, b.get_limb(i), zero_u32);
        ret = BoolTarget::new_unsafe(builder.select(is_zero, ret.target, false_t));
    }

    ret
}

pub fn get_balance_from_leaf<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: [BoolTarget; 256],
    balance_index: BigUintTarget,
) -> BigUintTarget {
    let balances_in_leaf = split_into_chunks(builder, leaf);
    let mut accumulator = ssz_num_from_bits(builder, &balances_in_leaf[0].clone());
    for i in 1..balances_in_leaf.len() {
        let current_index_t = builder.constant_biguint(&BigUint::from(i as u32));
        let current_balance_in_leaf = ssz_num_from_bits(builder, &balances_in_leaf[i].clone());

        let selector_enabled = biguint_is_equal(builder, &current_index_t, &balance_index);
        accumulator = select_biguint_target(
            builder,
            selector_enabled,
            current_balance_in_leaf,
            accumulator,
        );
    }

    accumulator
}

pub fn split_into_chunks<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: [BoolTarget; 256],
) -> [[BoolTarget; 64]; 4] {
    let mut chunks = [[builder._false(); 64]; 4];
    for (i, chunk) in chunks.iter_mut().enumerate() {
        chunk.copy_from_slice(&leaf[i * 64..(i + 1) * 64]);
    }
    chunks
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

pub fn create_bool_target_array<F: RichField + Extendable<D>, const D: usize>(
    //Stefan TODO: size of slice should be function parameter
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
    (0..ETH_SHA256_BIT_SIZE)
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

pub fn if_biguint<F: RichField + Extendable<D>, const D: usize>(
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

pub fn bytes_to_bools(bytes: &[u8]) -> Vec<bool> {
    let mut bool_values = Vec::new();

    for value in bytes {
        for i in (0..8).rev() {
            let mask = 1 << i;
            bool_values.push(value & mask != 0);
        }
    }

    bool_values
}

pub fn bools_to_bytes(bools: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = 0u8;

    for (index, bit) in bools.iter().enumerate() {
        if *bit {
            byte |= 1 << (7 - (index % 8));
        }

        if index % 8 == 7 {
            bytes.push(byte);
            byte = 0;
        }
    }

    if bools.len() % 8 != 0 {
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

        let bool_values = bytes_to_bools(values);

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
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::{
            target::BoolTarget,
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use serde::Deserialize;
    use std::{fs, iter::repeat, println};

    use crate::{biguint::CircuitBuilderBiguint, utils::ssz_num_from_bits};

    use super::{get_balance_from_leaf, get_validator_relevance};

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

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;

    #[test]
    fn test_get_validator_relevance() -> Result<()> {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
        let activation_epoch = builder.constant_biguint(&BigUint::from(28551 as u32));
        let current_epoch = builder.constant_biguint(&BigUint::from(285512 as u32));
        let withdrawable_epoch = builder.constant_biguint(&BigUint::from(2855125512 as u32));
        let is_validator_relevant = get_validator_relevance(
            &mut builder,
            &activation_epoch,
            &current_epoch,
            &withdrawable_epoch,
        );

        builder.assert_one(is_validator_relevant.target);

        let pw = PartialWitness::new();
        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn test_get_balance_from_leaf() -> Result<()> {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
        let leaf: [BoolTarget; 256] = [
            0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0,
            0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1,
            0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
        .iter()
        .map(|f| BoolTarget::new_unsafe(builder.constant(F::from_canonical_u8(*f))))
        .collect_vec()
        .try_into()
        .unwrap();
        let balance_index_0 = builder.zero_biguint();
        let balance_index_1 = builder.constant_biguint(&BigUint::from(1 as u32));
        let balance_index_2 = builder.constant_biguint(&BigUint::from(2 as u32));
        let balance_index_3 = builder.constant_biguint(&BigUint::from(3 as u32));
        let balance_from_leaf_at_index_0 =
            get_balance_from_leaf(&mut builder, leaf, balance_index_0);
        let balance_from_leaf_at_index_1 =
            get_balance_from_leaf(&mut builder, leaf, balance_index_1);
        let balance_from_leaf_at_index_2 =
            get_balance_from_leaf(&mut builder, leaf, balance_index_2);
        let balance_from_leaf_at_index_3 =
            get_balance_from_leaf(&mut builder, leaf, balance_index_3);

        let expected_balance_at_index_0 =
            builder.constant_biguint(&BigUint::from(32000579388 as u64));
        let expected_balance_at_index_1 =
            builder.constant_biguint(&BigUint::from(32000574671 as u64));
        let expected_balance_at_index_2 =
            builder.constant_biguint(&BigUint::from(32000579312 as u64));
        let expected_balance_at_index_3 =
            builder.constant_biguint(&BigUint::from(32000581683 as u64));

        builder.connect_biguint(&balance_from_leaf_at_index_0, &expected_balance_at_index_0);
        builder.connect_biguint(&balance_from_leaf_at_index_1, &expected_balance_at_index_1);
        builder.connect_biguint(&balance_from_leaf_at_index_2, &expected_balance_at_index_2);
        builder.connect_biguint(&balance_from_leaf_at_index_3, &expected_balance_at_index_3);

        let pw = PartialWitness::new();
        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
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
