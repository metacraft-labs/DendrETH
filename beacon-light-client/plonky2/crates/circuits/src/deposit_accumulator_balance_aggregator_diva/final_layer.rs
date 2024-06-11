use crate::{
    common_targets::{PoseidonMerkleBranchTarget, Sha256MerkleBranchTarget},
    pubkey_commitment_mapper::first_level::PubkeyCommitmentMapperFL,
    serializers::{serde_bool_array_to_hex_string, serde_bool_array_to_hex_string_nested},
    utils::circuit::{
        assert_slot_is_in_epoch::assert_slot_is_in_epoch,
        bits_to_bytes_target,
        hashing::{
            merkle::{
                poseidon::assert_merkle_proof_is_valid_const_poseidon,
                sha256::assert_merkle_proof_is_valid_const_sha256,
            },
            sha256::sha256,
        },
        target_to_be_bits, verify_proof,
    },
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
};
use circuit::{
    serde::serde_u64_str, targets::uint::Uint64Target, Circuit, CircuitInputTarget,
    CircuitOutputTarget, SSZHashTreeRoot,
};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOut, RichField},
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputsTarget,
    },
};

use crate::common_targets::Sha256Target;

use super::first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel;

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorDivaFinalLayerTarget {
    pub balance_aggregation_proof: ProofWithPublicInputsTarget<2>,
    pub validators_commitment_mapper_root_proof: ProofWithPublicInputsTarget<2>,
    pub validators_commitment_mapper_65536gindex_proof: ProofWithPublicInputsTarget<2>,
    pub pubkey_commitment_mapper_proof: ProofWithPublicInputsTarget<2>,

    // Public input
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub block_root: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub state_root: Sha256Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub state_root_branch: Sha256MerkleBranchTarget<3>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub validators_branch: Sha256MerkleBranchTarget<6>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balance_branch: Sha256MerkleBranchTarget<22>,

    // Public input
    #[target(in)]
    #[serde(with = "serde_u64_str")]
    pub execution_block_number: Uint64Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub execution_block_number_branch: Sha256MerkleBranchTarget<10>,

    // Public input
    #[target(in)]
    #[serde(with = "serde_u64_str")]
    pub slot: Uint64Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub slot_branch: Sha256MerkleBranchTarget<5>,

    #[target(out)]
    pub public_inputs_hash_bytes: [Target; 32],
}

pub struct DepositAccumulatorBalanceAggregatorDivaFinalLayer;

impl Circuit for DepositAccumulatorBalanceAggregatorDivaFinalLayer {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = DepositAccumulatorBalanceAggregatorDivaFinalLayerTarget;

    type Params = (
        CircuitData<Self::F, Self::C, { Self::D }>,
        CircuitData<Self::F, Self::C, { Self::D }>,
        CircuitData<Self::F, Self::C, { Self::D }>,
        CircuitData<Self::F, Self::C, { Self::D }>,
    );

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        (
            deposit_accumulator_balance_aggregator_circuit_data,
            validators_commitment_mapper_root_circuit_data,
            validators_commitment_mapper_65536_circuit_data,
            pubkey_commitment_mapper_circuit_data,
        ): &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let balance_aggregation_proof = verify_proof(
            builder,
            &deposit_accumulator_balance_aggregator_circuit_data,
        );
        let validators_commitment_mapper_root_proof =
            verify_proof(builder, &validators_commitment_mapper_root_circuit_data);
        let validators_commitment_mapper_65536gindex_proof =
            verify_proof(builder, &validators_commitment_mapper_65536_circuit_data);
        let pubkey_commitment_mapper_proof =
            verify_proof(builder, &pubkey_commitment_mapper_circuit_data);

        let balance_aggregation_pis =
            DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs_target(
                &balance_aggregation_proof.public_inputs,
            );
        let validators_commitment_mapper_root_pis =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
                &validators_commitment_mapper_root_proof.public_inputs,
            );
        let validators_commitment_mapper_65536gindex_pis =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
                &validators_commitment_mapper_65536gindex_proof.public_inputs,
            );

        let pubkey_commitment_mapper_pis = PubkeyCommitmentMapperFL::read_public_inputs_target(
            &pubkey_commitment_mapper_proof.public_inputs,
        );

        let poseidon_branch = get_validators_poseidon_branch(builder);

        assert_merkle_proof_is_valid_const_poseidon(
            builder,
            &validators_commitment_mapper_65536gindex_pis.poseidon_hash_tree_root,
            &validators_commitment_mapper_root_pis.poseidon_hash_tree_root,
            &poseidon_branch,
            65536,
        );

        builder.connect_hashes(
            validators_commitment_mapper_65536gindex_pis.poseidon_hash_tree_root,
            balance_aggregation_pis.validators_commitment_mapper_root,
        );
        builder.connect_hashes(
            pubkey_commitment_mapper_pis.poseidon,
            balance_aggregation_pis.pubkey_commitment_mapper_root,
        );

        validate_data_against_block_root(
            builder,
            &input,
            &balance_aggregation_pis.balances_root,
            &validators_commitment_mapper_root_pis.sha256_hash_tree_root,
        );

        assert_slot_is_in_epoch(builder, input.slot, balance_aggregation_pis.current_epoch);

        let mut public_inputs_hash = hash_public_inputs(
            builder,
            &input,
            &balance_aggregation_pis,
            &pubkey_commitment_mapper_pis,
        );

        // Mask the last 3 bits in big endian as zero
        public_inputs_hash[0] = builder._false();
        public_inputs_hash[1] = builder._false();
        public_inputs_hash[2] = builder._false();

        let public_inputs_hash_bytes = bits_to_bytes_target(builder, &public_inputs_hash)
            .try_into()
            .unwrap();

        Self::Target {
            balance_aggregation_proof,
            validators_commitment_mapper_root_proof,
            validators_commitment_mapper_65536gindex_proof,
            pubkey_commitment_mapper_proof,
            block_root: input.block_root,
            state_root: input.state_root,
            state_root_branch: input.state_root_branch,
            validators_branch: input.validators_branch,
            balance_branch: input.balance_branch,
            execution_block_number: input.execution_block_number,
            execution_block_number_branch: input.execution_block_number_branch,
            slot: input.slot,
            slot_branch: input.slot_branch,
            public_inputs_hash_bytes,
        }
    }
}

fn validate_data_against_block_root<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    input: &CircuitInputTarget<DepositAccumulatorBalanceAggregatorDivaFinalLayer>,
    balances_root_level_22: &Sha256Target,
    validators_root_left: &Sha256Target,
) {
    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &input.state_root,
        &input.block_root,
        &input.state_root_branch,
        11,
    );

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &validators_root_left,
        &input.state_root,
        &input.validators_branch,
        86,
    );

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &balances_root_level_22,
        &input.state_root,
        &input.balance_branch,
        5767168,
    );

    let slot_ssz = input.slot.ssz_hash_tree_root(builder);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &slot_ssz,
        &input.state_root,
        &input.slot_branch,
        34,
    );

    let block_number_ssz = input.execution_block_number.ssz_hash_tree_root(builder);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &block_number_ssz,
        &input.state_root,
        &input.execution_block_number_branch,
        1798,
    );
}

fn hash_public_inputs<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    input: &CircuitInputTarget<DepositAccumulatorBalanceAggregatorDivaFinalLayer>,
    balance_aggregation_pis: &CircuitOutputTarget<
        DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    >,
    pubkey_commitment_mapper_pis: &CircuitOutputTarget<PubkeyCommitmentMapperFL>,
) -> Sha256Target {
    let balance_bits = balance_aggregation_pis
        .accumulated_data
        .balance
        .to_be_bits(builder);

    let block_number_bits = input.execution_block_number.to_be_bits(builder);

    let number_of_non_activated_validators_bits = target_to_be_bits(
        builder,
        balance_aggregation_pis
            .accumulated_data
            .validator_status_stats
            .non_activated_count,
    );
    let number_of_active_validators_bits = target_to_be_bits(
        builder,
        balance_aggregation_pis
            .accumulated_data
            .validator_status_stats
            .active_count,
    );
    let number_of_exited_validators_bits = target_to_be_bits(
        builder,
        balance_aggregation_pis
            .accumulated_data
            .validator_status_stats
            .exited_count,
    );
    let number_of_slashed_validators_bits = target_to_be_bits(
        builder,
        balance_aggregation_pis
            .accumulated_data
            .validator_status_stats
            .slashed_count,
    );

    sha256(
        builder,
        &[
            input.block_root.as_slice(),
            block_number_bits.as_slice(),
            pubkey_commitment_mapper_pis.sha256.as_slice(),
            balance_bits.as_slice(),
            number_of_non_activated_validators_bits.as_slice(),
            number_of_active_validators_bits.as_slice(),
            number_of_exited_validators_bits.as_slice(),
            number_of_slashed_validators_bits.as_slice(),
        ]
        .concat(),
    )
}

fn get_validators_poseidon_branch(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
) -> PoseidonMerkleBranchTarget<16> {
    [
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(3896366420105793420),
                GoldilocksField(17410332186442776169),
                GoldilocksField(7329967984378645716),
                GoldilocksField(6310665049578686403),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(6574146240104132812),
                GoldilocksField(2239043898123515337),
                GoldilocksField(13809601679688051486),
                GoldilocksField(16196448971140258304),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(7429917014148897946),
                GoldilocksField(13764740161233226515),
                GoldilocksField(14310941960777962392),
                GoldilocksField(10321132974520710857),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(16852763145767657080),
                GoldilocksField(5650551567722662817),
                GoldilocksField(4688637260797538488),
                GoldilocksField(504212361217900660),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(17594730245457333136),
                GoldilocksField(13719209718183388763),
                GoldilocksField(11444947689050098668),
                GoldilocksField(628489339233491445),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(7731246070744876899),
                GoldilocksField(3033565575746121792),
                GoldilocksField(14735263366152051322),
                GoldilocksField(16212144996433476818),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(9947841139978160787),
                GoldilocksField(692236217135079542),
                GoldilocksField(16309341595179079658),
                GoldilocksField(9294006745033445642),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(8603459983426387388),
                GoldilocksField(1706773463182378335),
                GoldilocksField(10020230853197995171),
                GoldilocksField(2362856042482390481),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(16463394126558395459),
                GoldilocksField(12818610997234032270),
                GoldilocksField(2968763245313636978),
                GoldilocksField(15445927884703223427),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(16924929798993045119),
                GoldilocksField(9228476078763095559),
                GoldilocksField(3639599968030750173),
                GoldilocksField(9842693474971302918),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(2488667422532942441),
                GoldilocksField(619530082608543022),
                GoldilocksField(3698308124541679027),
                GoldilocksField(1337151890861372088),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(10420632113085830027),
                GoldilocksField(2043024317550638523),
                GoldilocksField(9353702824282721936),
                GoldilocksField(13923517817060358740),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(2864602688424687291),
                GoldilocksField(3849603923476837883),
                GoldilocksField(15617889861797529219),
                GoldilocksField(12429234418051645329),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(2558543962574772915),
                GoldilocksField(9272315342420626056),
                GoldilocksField(4474448392614911585),
                GoldilocksField(1483027055753170828),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(15131845414406822716),
                GoldilocksField(5979581984005702075),
                GoldilocksField(6999690762874000865),
                GoldilocksField(9727258862093954055),
            ],
        }),
        builder.constant_hash(HashOut {
            elements: [
                GoldilocksField(16947881275436717432),
                GoldilocksField(7978417559450660789),
                GoldilocksField(5545004785373663100),
                GoldilocksField(8368806924824039910),
            ],
        }),
    ]
}
