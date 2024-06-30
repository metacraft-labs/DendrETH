use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions,
    targets::uint::{
        ops::{arithmetic::Zero, comparison::EqualTo},
        Uint64Target,
    },
};
use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitData,
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::{biguint::BigUintTarget, u32::arithmetic_u32::U32Target};

use crate::common_targets::SSZTarget;

pub mod assert_slot_is_in_epoch;
pub mod hashing;
pub mod validator_status;

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

pub fn biguint_target_from_le_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits: &[BoolTarget],
) -> BigUintTarget {
    let bit_len = bits.len();
    assert_eq!(bit_len % 32, 0);

    let mut u32_targets = Vec::new();
    for i in 0..bit_len / 32 {
        u32_targets.push(U32Target(
            builder.le_sum(bits[i * 32..(i + 1) * 32].iter().rev()),
        ));
    }
    u32_targets.reverse();
    BigUintTarget { limbs: u32_targets }
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

pub fn target_to_be_bits<F: RichField + Extendable<D>, const D: usize>(
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

fn split_into_chunks(leaf: &[BoolTarget; 256]) -> [[BoolTarget; 64]; 4] {
    let mut chunks = Vec::new();

    for i in 0..4 {
        chunks.push(leaf[i * 64..(i + 1) * 64].try_into().unwrap());
    }

    chunks.try_into().unwrap()
}

/// `balance_index` must be in the range [0, 3].
pub fn get_balance_from_leaf<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &SSZTarget,
    balance_index: Uint64Target,
) -> Uint64Target {
    let balances_in_leaf = split_into_chunks(leaf);
    let mut accumulator = Uint64Target::zero(builder);
    for i in 0..balances_in_leaf.len() {
        let current_index_t = Uint64Target::constant(i as u64, builder);
        let current_balance_in_leaf = Uint64Target::from_le_bytes(&balances_in_leaf[i], builder);

        let selector_enabled = current_index_t.equal_to(balance_index, builder);

        accumulator =
            builder.select_target(selector_enabled, &current_balance_in_leaf, &accumulator);
    }

    accumulator
}

#[cfg(test)]
mod test_ssz_num_from_bits {
    use anyhow::Result;
    use circuit::{
        circuit_builder_extensions::CircuitBuilderExtensions,
        targets::uint::{
            ops::{arithmetic::Zero, comparison::EqualTo},
            Uint64Target,
        },
    };
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

            if num_bits != 64 {
                // For now lets test only test 64 bits
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

                let target = Uint64Target::from_le_bytes(&bits, &mut builder);

                let value = test_case.value.parse::<u64>().expect(
                    format!(
                        "Unable to parse value: {}_{}",
                        test_case.r#type, test_case.tags[2]
                    )
                    .as_str(),
                );

                let expected_target = Uint64Target::constant(value, &mut builder);
                let values_are_equal = target.equal_to(expected_target, &mut builder);
                builder.assert_true(values_are_equal);

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

        let balance_index_0 = Uint64Target::zero(&mut builder);
        let balance_index_1 = Uint64Target::constant(1, &mut builder);
        let balance_index_2 = Uint64Target::constant(2, &mut builder);
        let balance_index_3 = Uint64Target::constant(3, &mut builder);

        let balance_from_leaf_at_index_0 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_0);
        let balance_from_leaf_at_index_1 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_1);
        let balance_from_leaf_at_index_2 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_2);
        let balance_from_leaf_at_index_3 =
            get_balance_from_leaf(&mut builder, &leaf, balance_index_3);

        let expected_balance_at_index_0 = Uint64Target::constant(32000579388, &mut builder);
        let expected_balance_at_index_1 = Uint64Target::constant(32000574671, &mut builder);
        let expected_balance_at_index_2 = Uint64Target::constant(32000579312, &mut builder);
        let expected_balance_at_index_3 = Uint64Target::constant(32000581683, &mut builder);

        builder
            .assert_targets_are_equal(&balance_from_leaf_at_index_0, &expected_balance_at_index_0);
        builder
            .assert_targets_are_equal(&balance_from_leaf_at_index_1, &expected_balance_at_index_1);
        builder
            .assert_targets_are_equal(&balance_from_leaf_at_index_2, &expected_balance_at_index_2);
        builder
            .assert_targets_are_equal(&balance_from_leaf_at_index_3, &expected_balance_at_index_3);

        let pw = PartialWitness::new();
        let data = builder.build::<PoseidonGoldilocksConfig>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }
}
