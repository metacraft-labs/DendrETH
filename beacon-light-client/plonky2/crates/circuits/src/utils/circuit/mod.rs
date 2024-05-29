use circuit::circuit_builder_extensions::CircuitBuilderExtensions;
use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    u32::arithmetic_u32::U32Target,
};

use crate::common_targets::SSZTarget;

use self::hashing::merkle::ssz::ssz_num_from_bits;

pub mod hashing;

pub fn verify_proof<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    circuit_data: &CircuitData<F, C, D>,
) -> ProofWithPublicInputsTarget<D>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let proof = builder.add_virtual_proof_with_pis(&circuit_data.common);
    let verifier_circuit_data = builder.constant_verifier_data(&circuit_data.verifier_only);
    builder.verify_proof::<C>(&proof, &verifier_circuit_data, &circuit_data.common);
    proof
}

pub fn connect_arrays<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[Target],
    second: &[Target],
) {
    assert!(first.len() == second.len());

    for idx in 0..first.len() {
        builder.connect(first[idx], second[idx]);
    }
}

pub fn connect_bool_arrays<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[BoolTarget],
    second: &[BoolTarget],
) {
    let first = first
        .iter()
        .map(|bool_target| bool_target.target)
        .collect_vec();
    let second = second
        .iter()
        .map(|bool_target| bool_target.target)
        .collect_vec();
    connect_arrays(builder, first.as_slice(), second.as_slice())
}

pub fn bool_arrays_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[BoolTarget],
    second: &[BoolTarget],
) -> BoolTarget {
    let first = first
        .iter()
        .map(|bool_target| bool_target.target)
        .collect_vec();
    let second = second
        .iter()
        .map(|bool_target| bool_target.target)
        .collect_vec();
    arrays_are_equal(builder, first.as_slice(), second.as_slice())
}

pub fn assert_bool_arrays_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[BoolTarget],
    second: &[BoolTarget],
) {
    let are_equal = bool_arrays_are_equal(builder, first, second);
    builder.assert_true(are_equal);
}

pub fn assert_arrays_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[Target],
    second: &[Target],
) {
    let are_equal = arrays_are_equal(builder, first, second);
    builder.assert_true(are_equal);
}

pub fn arrays_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[Target],
    second: &[Target],
) -> BoolTarget {
    assert!(first.len() == second.len());

    let mut result = builder._true();
    for idx in 0..first.len() {
        let is_equal = builder.is_equal(first[idx], second[idx]);
        result = builder.and(result, is_equal);
    }
    result
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

pub fn bits_to_bytes_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits: &[BoolTarget],
) -> Vec<Target> {
    assert!(bits.len() % 8 == 0);
    bits.chunks(8)
        .map(|byte_bits| builder.le_sum(byte_bits.iter().rev()))
        .collect_vec()
}

pub fn reverse_endianness(bits: &[BoolTarget]) -> Vec<BoolTarget> {
    bits.chunks(8).rev().flatten().cloned().collect()
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
pub fn is_equal_u32<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: U32Target,
    y: U32Target,
) -> BoolTarget {
    builder.is_equal(x.0, y.0)
}

pub fn biguint_is_equal_non_equal_limbs<F: RichField + Extendable<D>, const D: usize>(
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

pub fn split_into_chunks(leaf: &[BoolTarget; 256]) -> [[BoolTarget; 64]; 4] {
    let mut chunks = Vec::new();

    for i in 0..4 {
        chunks.push(leaf[i * 64..(i + 1) * 64].try_into().unwrap());
    }

    chunks.try_into().unwrap()
}

pub fn get_balance_from_leaf<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &SSZTarget,
    balance_index: BigUintTarget,
) -> BigUintTarget {
    let balances_in_leaf = split_into_chunks(leaf);
    let mut accumulator = ssz_num_from_bits(builder, &balances_in_leaf[0].clone());
    for i in 1..balances_in_leaf.len() {
        let current_index_t = builder.constant_biguint(&BigUint::from(i as u32));
        let current_balance_in_leaf = ssz_num_from_bits(builder, &balances_in_leaf[i].clone());

        let selector_enabled =
            biguint_is_equal_non_equal_limbs(builder, &current_index_t, &balance_index);
        accumulator = select_biguint(
            builder,
            selector_enabled,
            &current_balance_in_leaf,
            &accumulator,
        );
    }

    accumulator
}

pub fn biguint_target_from_limbs(limbs: &[Target]) -> BigUintTarget {
    BigUintTarget {
        limbs: limbs.iter().cloned().map(|x| U32Target(x)).collect_vec(),
    }
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
    use plonky2_crypto::biguint::CircuitBuilderBiguint;
    use serde::Deserialize;
    use std::{fs, iter::repeat, println};

    use crate::utils::circuit::hashing::merkle::ssz::ssz_num_from_bits;

    use super::get_balance_from_leaf;

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
        let bound_test_cases =
            get_test_cases("../../../../vendor/eth2.0-tests/ssz/uint_bounds.yaml")?
                .iter()
                .cloned()
                .filter(|x| x.valid)
                .collect_vec();

        let random_test_cases =
            get_test_cases("../../../../vendor/eth2.0-tests/ssz/uint_random.yaml")?
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

    #[test]
    fn test_get_balance_from_leaf() -> Result<()> {
        let mut builder =
            CircuitBuilder::<GoldilocksField, 2>::new(CircuitConfig::standard_recursion_config());
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
        .map(|f| BoolTarget::new_unsafe(builder.constant(GoldilocksField::from_canonical_u8(*f))))
        .collect_vec()
        .try_into()
        .unwrap();

        let balance_index_0 = builder.zero_biguint();
        let balance_index_1 = builder.constant_biguint(&BigUint::from(1 as u32));
        let balance_index_2 = builder.constant_biguint(&BigUint::from(2 as u32));
        let balance_index_3 = builder.constant_biguint(&BigUint::from(3 as u32));
        let balance_from_leaf_at_index_0 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_0);
        let balance_from_leaf_at_index_1 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_1);
        let balance_from_leaf_at_index_2 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_2);
        let balance_from_leaf_at_index_3 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_3);

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
        let data = builder.build::<PoseidonGoldilocksConfig>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }
}
