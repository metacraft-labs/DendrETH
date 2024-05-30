use crate::{
    common_targets::{PubkeyTarget, Sha256MerkleBranchTarget},
    deposits_accumulator_balance_aggregator::common_targets::ValidatorStatusStatsTarget,
    serializers::{
        biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
        serde_bool_array_to_hex_string_nested,
    },
    utils::circuit::{
        assert_bool_arrays_are_equal, get_balance_from_leaf,
        hashing::{
            merkle::{
                poseidon::{hash_validator_poseidon, validate_merkle_proof_poseidon},
                sha256::validate_merkle_proof_sha256,
            },
            poseidon::poseidon_or_zeroes,
        },
        select_biguint,
        validator_status::{get_validator_relevance, get_validator_status},
    },
};
use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit, ToTargets};
use circuit_derive::{CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::common_targets::{PoseidonMerkleBranchTarget, Sha256Target, ValidatorTarget};

pub struct DepositAccumulatorBalanceAggregatorDivaFirstLevel {}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatedDataTargetDiva {
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub balance: BigUintTarget,
    pub validator_status_stats: ValidatorStatusStatsTarget,
}

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorDivaFirstLevelTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub validators_commitment_mapper_branch: PoseidonMerkleBranchTarget<24>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub validator_gindex: BigUintTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub deposit_pubkey: PubkeyTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balance_leaf: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balance_branch: Sha256MerkleBranchTarget<22>,

    #[target(in)]
    pub is_dummy: BoolTarget,

    #[target(in, out)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUintTarget,

    #[target(in, out)]
    pub validators_commitment_mapper_root: HashOutTarget,

    #[target(in, out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balances_root: Sha256Target,

    #[target(out)]
    pub pubkey_commitment_mapper_root: HashOutTarget,

    #[target(out)]
    pub accumulated_data: AccumulatedDataTargetDiva,
}

impl Circuit for DepositAccumulatorBalanceAggregatorDivaFirstLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = DepositAccumulatorBalanceAggregatorDivaFirstLevelTarget;

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        _params: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let deposit_is_real = builder.not(input.is_dummy);

        let deposits_hash_tree_root =
            poseidon_or_zeroes(builder, input.deposit_pubkey.to_targets(), deposit_is_real);

        let validator_hash_tree_root = hash_validator_poseidon(builder, &input.validator);

        let is_valid = validate_merkle_proof_poseidon(
            builder,
            &validator_hash_tree_root,
            &input.validators_commitment_mapper_root,
            &input.validators_commitment_mapper_branch,
            &input.validator_gindex,
        );

        builder.assert_implication(deposit_is_real, is_valid);

        assert_bool_arrays_are_equal(builder, &input.validator.pubkey, &input.deposit_pubkey);

        let four = builder.constant_biguint(&BigUint::from_u64(4u64).unwrap());
        let balance_inner_index = builder.rem_biguint(&input.validator_gindex, &four);
        let balance = get_balance_from_leaf(builder, &input.balance_leaf, balance_inner_index);
        let balance_gindex = builder.div_biguint(&input.validator_gindex, &four);

        let is_valid = validate_merkle_proof_sha256(
            builder,
            &input.balance_leaf,
            &input.balances_root,
            &input.balance_branch,
            &balance_gindex,
        );

        builder.assert_implication(deposit_is_real, is_valid);

        let (is_non_activated, is_active, is_exited) = get_validator_status(
            builder,
            &input.validator.activation_epoch,
            &input.current_epoch,
            &input.validator.exit_epoch,
        );

        let zero_validator_status_stats: ValidatorStatusStatsTarget = builder.zero_init();
        let mut validator_status_stats = ValidatorStatusStatsTarget {
            non_activated_count: is_non_activated.target,
            active_count: is_active.target,
            exited_count: is_exited.target,
            slashed_count: input.validator.slashed.target,
        };
        validator_status_stats = builder.select_target(
            deposit_is_real,
            &validator_status_stats,
            &zero_validator_status_stats,
        );

        let is_relevant = get_validator_relevance(
            builder,
            &input.validator.activation_epoch,
            &input.current_epoch,
            &input.validator.withdrawable_epoch,
        );

        let zero_biguint = builder.zero_biguint();
        let validator_balance = select_biguint(builder, is_relevant, &balance, &zero_biguint);

        let zero_accumulated_data: AccumulatedDataTargetDiva = builder.zero_init();
        let mut accumulated_data = AccumulatedDataTargetDiva {
            balance: validator_balance,
            validator_status_stats,
        };
        accumulated_data =
            builder.select_target(deposit_is_real, &accumulated_data, &zero_accumulated_data);

        Self::Target {
            accumulated_data,

            validator: input.validator,
            validators_commitment_mapper_root: input.validators_commitment_mapper_root,
            validators_commitment_mapper_branch: input.validators_commitment_mapper_branch,
            validator_gindex: input.validator_gindex,
            deposit_pubkey: input.deposit_pubkey,
            balances_root: input.balances_root,
            balance_leaf: input.balance_leaf,
            balance_branch: input.balance_branch,
            is_dummy: input.is_dummy,
            current_epoch: input.current_epoch,
            pubkey_commitment_mapper_root: deposits_hash_tree_root,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{self, Read},
        time::Instant,
    };

    use circuit::{Circuit, CircuitInput, SetWitness};
    use num::{BigUint, FromPrimitive};
    use plonky2::{field::goldilocks_field::GoldilocksField, iop::witness::PartialWitness};

    use crate::utils::bytes_to_bits;

    use super::DepositAccumulatorBalanceAggregatorDivaFirstLevel;

    const INPUT_FILE: &str = "src/deposit_accumulator_balance_aggregator_diva/deposit_accumulator_balance_aggregator_diva_input.json";

    #[test]
    pub fn test_deposit_accumulator_diva_leaf_circuit_valid() {
        let json_str = read_file_to_string().unwrap();

        let json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );

        assert_eq!(result.current_epoch, BigUint::from_u64(158342).unwrap());
        assert_eq!(
            result.validators_commitment_mapper_root.0,
            [
                8778336758134959946,
                14486974390460197235,
                4868457073824047267,
                16603036372618420521
            ],
        );

        let balance_root_bools: [bool; 256] = bytes_to_bits(
            &hex::decode("20fe0fb226a1c08e1830dfab419b67caea4f4d090b7b5a73e8b9c2439b60611d")
                .unwrap(),
        )
        .try_into()
        .unwrap();

        assert_eq!(result.balances_root.0, balance_root_bools);
        assert_eq!(
            result.accumulated_data.balance,
            BigUint::from_u64(31035128496).unwrap()
        );

        assert_eq!(
            result
                .accumulated_data
                .validator_status_stats
                .non_activated_count,
            0
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.active_count,
            1
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.exited_count,
            0
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.slashed_count,
            0
        );
        assert_ne!(result.pubkey_commitment_mapper_root.0, [0, 0, 0, 0]);
    }

    #[test]
    #[should_panic]
    pub fn test_deposit_accumulator_diva_leaf_circuit_different_pubkeys() {
        let json_str = read_file_to_string().unwrap();
        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.deposit_pubkey[253] = true;

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let _result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );
    }

    #[test]
    #[should_panic]
    pub fn test_deposit_accumulator_diva_leaf_circuit_wrong_commitment_mapper_branch() {
        let json_str = read_file_to_string().unwrap();
        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.validators_commitment_mapper_branch[0].0[0] = 14253833605643055269;

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let _result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );
    }

    #[test]
    #[should_panic]
    pub fn test_deposit_accumulator_diva_leaf_circuit_wrong_balances_root() {
        let json_str = read_file_to_string().unwrap();
        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.balance_branch[0].0[123] = true;

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let _result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );
    }

    #[test]
    #[should_panic]
    pub fn test_deposit_accumulator_diva_leaf_circuit_wrong_validator() {
        let json_str = read_file_to_string().unwrap();
        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.validator.activation_epoch = BigUint::from_u64(817).unwrap();

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let _result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );
    }

    #[test]
    #[should_panic]
    pub fn test_deposit_accumulator_diva_leaf_circuit_wrong_validator_gindex() {
        let json_str = read_file_to_string().unwrap();
        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.validator_gindex = BigUint::from_u64(817).unwrap();

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let _result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );
    }

    #[test]
    pub fn test_deposit_accumulator_diva_leaf_circuit_is_dummy() {
        let json_str = read_file_to_string().unwrap();

        let mut json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
        >(&json_str)
        .unwrap();

        json_input.is_dummy = true;

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());
        println!("Circuit built in {:?}", s.elapsed());

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let result = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &proof.public_inputs,
        );

        assert_eq!(result.current_epoch, BigUint::from_u64(158342).unwrap());
        assert_eq!(
            result.validators_commitment_mapper_root.0,
            [
                8778336758134959946,
                14486974390460197235,
                4868457073824047267,
                16603036372618420521
            ],
        );

        let balance_root_bools: [bool; 256] = bytes_to_bits(
            &hex::decode("20fe0fb226a1c08e1830dfab419b67caea4f4d090b7b5a73e8b9c2439b60611d")
                .unwrap(),
        )
        .try_into()
        .unwrap();

        assert_eq!(result.balances_root.0, balance_root_bools);
        assert_eq!(
            result.accumulated_data.balance,
            BigUint::from_u64(0).unwrap()
        );

        assert_eq!(
            result
                .accumulated_data
                .validator_status_stats
                .non_activated_count,
            0
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.active_count,
            0
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.exited_count,
            0
        );
        assert_eq!(
            result.accumulated_data.validator_status_stats.slashed_count,
            0
        );

        assert_eq!(result.pubkey_commitment_mapper_root.0, [0, 0, 0, 0]);
    }

    fn read_file_to_string() -> io::Result<String> {
        let mut file = File::open(INPUT_FILE)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}
