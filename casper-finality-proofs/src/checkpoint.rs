use plonky2x::{
    frontend::vars::SSZVariable,
    prelude::{Bytes32Variable, CircuitVariable, U64Variable},
};
use serde::Deserialize;
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub struct CheckpointVariable {
    pub epoch: U64Variable,
    pub root: Bytes32Variable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Checkpoint {
    pub epoch: u64,
    pub root: String,
}

// TODO: implement
impl CircuitVariable for CheckpointVariable {
    type ValueType<F: plonky2x::prelude::RichField> = Checkpoint;

    fn init_unsafe<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
    ) -> Self {
        todo!()
    }

    fn variables(&self) -> Vec<plonky2x::prelude::Variable> {
        todo!()
    }

    fn from_variables_unsafe(variables: &[plonky2x::prelude::Variable]) -> Self {
        todo!()
    }

    fn assert_is_valid<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
    ) {
        todo!()
    }

    fn elements<F: plonky2x::prelude::RichField>(value: Self::ValueType<F>) -> Vec<F> {
        todo!()
    }

    fn from_elements<F: plonky2x::prelude::RichField>(elements: &[F]) -> Self::ValueType<F> {
        todo!()
    }

    fn init<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
    ) -> Self {
        let variable = Self::init_unsafe(builder);
        variable.assert_is_valid(builder);
        variable
    }

    fn constant<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
        value: Self::ValueType<L::Field>,
    ) -> Self {
        let field_elements = Self::elements::<L::Field>(value);
        let variables = field_elements
            .into_iter()
            .map(|element| builder.constant::<plonky2x::prelude::Variable>(element))
            .collect_vec();
        // Because this is a constant, we do not need to add constraints to ensure validity
        // as it is assumed that the value is valid.
        Self::from_variables_unsafe(&variables)
    }

    fn from_variables<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
        variables: &[plonky2x::prelude::Variable],
    ) -> Self {
        let variable = Self::from_variables_unsafe(variables);
        variable.assert_is_valid(builder);
        variable
    }

    fn get<F: plonky2x::prelude::RichField, W: plonky2x::prelude::Witness<F>>(
        &self,
        witness: &W,
    ) -> Self::ValueType<F> {
        let target_values = self
            .targets()
            .into_iter()
            .map(|t| witness.get_target(t))
            .collect::<Vec<F>>();
        Self::from_elements::<F>(&target_values)
    }

    fn set<F: plonky2x::prelude::RichField, W: plonky2x::prelude::WitnessWrite<F>>(
        &self,
        witness: &mut W,
        value: Self::ValueType<F>,
    ) {
        let elements = Self::elements::<F>(value);
        let targets = self.targets();
        assert_eq!(elements.len(), targets.len());
        for (element, target) in elements.into_iter().zip(targets.into_iter()) {
            witness.set_target(target, element);
        }
    }

    fn targets(&self) -> Vec<plonky2x::prelude::Target> {
        self.variables().into_iter().map(|v| v.0).collect()
    }

    fn from_targets(targets: &[plonky2x::prelude::Target]) -> Self {
        Self::from_variables_unsafe(
            &targets
                .iter()
                .map(|t| plonky2x::prelude::Variable(*t))
                .collect_vec(),
        )
    }

    fn nb_elements() -> usize {
        type L = plonky2x::prelude::DefaultParameters;
        const D: usize = 2;
        plonky2x::utils::disable_logging();
        let mut builder = plonky2x::prelude::CircuitBuilder::<L, D>::new();
        let variable = builder.init_unsafe::<Self>();
        plonky2x::utils::enable_logging();
        variable.variables().len()
    }
}

impl SSZVariable for CheckpointVariable {
    fn hash_tree_root<L: plonky2x::prelude::PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut plonky2x::prelude::CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let epoch_leaf = self.epoch.hash_tree_root(builder);
        let root_leaf = self.root.hash_tree_root(builder);

        builder.sha256_pair(epoch_leaf, root_leaf)
    }
}

// TODO: test
