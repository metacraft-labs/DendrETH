use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild},
    frontend::{
        eth::beacon::vars::BeaconValidatorVariable,
        hash::poseidon::poseidon256::PoseidonHashOutVariable, vars::EvmVariable,
    },
    prelude::{
        ArrayVariable, BoolVariable, Bytes32Variable, CircuitBuilder, PlonkParameters,
        U256Variable, U64Variable,
    },
};

use crate::{
    commitment_mapper_variable::{poseidon_hash_tree_root_leafs, CommitmentMapperVariable},
    proof_utils::ProofWithPublicInputsTargetReader,
};

pub fn define_get_total_active_balance_inner_level<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    child_circuit: &CircuitBuild<L, D>,
) where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
    let proof1 = builder.proof_read(&child_circuit).into();
    let proof2 = builder.proof_read(&child_circuit).into();

    builder.verify_proof::<L>(&proof1, &verifier_data, &child_circuit.data.common);
    builder.verify_proof::<L>(&proof2, &verifier_data, &child_circuit.data.common);

    let mut proof1_reader = ProofWithPublicInputsTargetReader::from(proof1);
    let mut proof2_reader = ProofWithPublicInputsTargetReader::from(proof2);

    let balances_root1 = proof1_reader.read::<Bytes32Variable>();
    let validators_root1 = proof1_reader.read::<PoseidonHashOutVariable>();
    let current_epoch1 = proof1_reader.read::<U256Variable>();
    let sum1 = proof1_reader.read::<U64Variable>();

    let balances_root2 = proof2_reader.read::<Bytes32Variable>();
    let validators_root2 = proof2_reader.read::<PoseidonHashOutVariable>();
    let current_epoch2 = proof2_reader.read::<U256Variable>();
    let sum2 = proof2_reader.read::<U64Variable>();

    builder.assert_is_equal(current_epoch1, current_epoch2);

    let validators_root = builder.poseidon_hash_pair(validators_root1, validators_root2);
    let balances_root = builder.sha256_pair(balances_root1, balances_root2);
    let sum = builder.add(sum1, sum2);

    builder.proof_write(sum);
    builder.proof_write(current_epoch1);
    builder.proof_write(validators_root);
    builder.proof_write(balances_root);
}

#[derive(Debug, Clone)]
pub struct TotalActiveBalanceInnerLevel;

impl Circuit for TotalActiveBalanceInnerLevel {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let validators = builder.read::<ArrayVariable<BeaconValidatorVariable, 8>>();
        let balances_leaves = builder.read::<ArrayVariable<Bytes32Variable, 2>>();

        let balances_root = builder.ssz_hash_leafs(balances_leaves.as_slice());

        let mut validators_leaves = Vec::new();

        for i in 0..8 {
            validators_leaves.push(CommitmentMapperVariable::hash_tree_root(
                &validators.data[i],
                builder,
            ));
        }

        let validators_hash_tree_root = poseidon_hash_tree_root_leafs(builder, &validators_leaves);

        let current_epoch = builder.read::<U256Variable>();

        let mut sum = builder.zero::<U64Variable>();

        for i in 0..8 {
            let balance = U64Variable::decode(
                builder,
                &balances_leaves.data[i / 4].0 .0[i % 4 * 8..i % 4 * 8 + 8],
            );

            let is_active = is_active_validator(builder, validators.data[i], current_epoch);

            let zero = builder.zero::<U64Variable>();

            let current = builder.select(is_active, zero, balance);

            sum = builder.add(sum, current);
        }

        builder.write(sum);
        builder.write(validators_hash_tree_root);
        builder.write(balances_root);
        builder.write(current_epoch);
    }
}

fn is_active_validator<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_validator: BeaconValidatorVariable,
    current_epoch: U256Variable,
) -> BoolVariable {
    let activation_epoch_lte_current_epoch =
        builder.lte(beacon_validator.activation_epoch, current_epoch);

    let current_epoch_lt_exit_epoch = builder.lt(current_epoch, beacon_validator.exit_epoch);

    builder.and(
        activation_epoch_lte_current_epoch,
        current_epoch_lt_exit_epoch,
    )
}
