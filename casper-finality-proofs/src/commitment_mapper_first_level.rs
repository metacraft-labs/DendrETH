use plonky2x::{
    backend::circuit::Circuit,
    frontend::{eth::beacon::vars::BeaconValidatorVariable, vars::SSZVariable},
    prelude::{CircuitBuilder, PlonkParameters},
};

use crate::{
    commitment_mapper_variable::{poseidon_hash_tree_root_leafs, CommitmentMapperVariable},
    validator::ValidatorVariable,
};

#[derive(Debug, Clone)]
pub struct CommitmentMapperFirstLevel;

impl Circuit for CommitmentMapperFirstLevel {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let validator = builder.read::<ValidatorVariable>();
        let validator1 = builder.read::<ValidatorVariable>();
        let validator2 = builder.read::<ValidatorVariable>();
        let validator3 = builder.read::<ValidatorVariable>();

        // let sha256_hash_tree_root = SSZVariable::hash_tree_root(&validator, builder);
        let poseidon_hash_tree_root = CommitmentMapperVariable::hash_tree_root(&validator, builder);
        let poseidon_hash_tree_root1 =
            CommitmentMapperVariable::hash_tree_root(&validator1, builder);
        let poseidon_hash_tree_root2 =
            CommitmentMapperVariable::hash_tree_root(&validator2, builder);
        let poseidon_hash_tree_root3 =
            CommitmentMapperVariable::hash_tree_root(&validator3, builder);

        let poseidon_hash_tree_root = poseidon_hash_tree_root_leafs(
            builder,
            &[
                poseidon_hash_tree_root,
                poseidon_hash_tree_root1,
                poseidon_hash_tree_root2,
                poseidon_hash_tree_root3,
            ],
        );

        // builder.write(sha256_hash_tree_root);
        builder.write(poseidon_hash_tree_root);
    }
}
