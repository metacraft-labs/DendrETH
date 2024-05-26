use itertools::Itertools;
use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig};
use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};
use plonky2_crypto::u32::arithmetic_u32::U32Target;

pub mod hashing;

pub fn create_verifier_circuit_target<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    verifier_only: &VerifierOnlyCircuitData<C, D>,
) -> VerifierCircuitTarget
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    VerifierCircuitTarget {
        constants_sigmas_cap: builder.constant_merkle_cap(&verifier_only.constants_sigmas_cap),
        circuit_digest: builder.constant_hash(verifier_only.circuit_digest),
    }
}

pub fn verify_proof<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    circuit_data: &CircuitData<F, C, D>,
) -> ProofWithPublicInputsTarget<D>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let proof = builder.add_virtual_proof_with_pis(&circuit_data.common);
    let verifier_circuit_data =
        create_verifier_circuit_target(builder, &circuit_data.verifier_only);
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

fn reverse_endianness(bits: &[BoolTarget]) -> Vec<BoolTarget> {
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
        field::goldilocks_field::GoldilocksField,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use plonky2_crypto::biguint::CircuitBuilderBiguint;
    use serde::Deserialize;
    use std::{fs, iter::repeat, println};

    use crate::utils::circuit::hashing::merkle::ssz::ssz_num_from_bits;

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
}
