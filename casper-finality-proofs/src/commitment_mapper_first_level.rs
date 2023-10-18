use plonky2x::{
    backend::circuit::Circuit,
    frontend::{eth::beacon::vars::BeaconValidatorVariable, vars::SSZVariable},
    prelude::{CircuitBuilder, PlonkParameters},
};

use crate::commitment_mapper_variable::CommitmentMapperVariable;

#[derive(Debug, Clone)]
pub struct CommitmentMapperFirstLevel;

impl Circuit for CommitmentMapperFirstLevel {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let validator = builder.read::<BeaconValidatorVariable>();

        let sha256_hash_tree_root = SSZVariable::hash_tree_root(&validator, builder);
        let poseidon_hash_tree_root = CommitmentMapperVariable::hash_tree_root(&validator, builder);

        builder.write(sha256_hash_tree_root);
        builder.write(poseidon_hash_tree_root);
    }
}
