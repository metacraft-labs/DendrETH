use plonky2::plonk::config::{AlgebraicHasher, GenericConfig};
use plonky2x::{
    frontend::{
        eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{ArrayVariable, CircuitBuilder, CircuitVariable, PlonkParameters, Variable},
};

pub trait CommitmentMapperVariable: CircuitVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> PoseidonHashOutVariable
    where
        <<L as PlonkParameters<D>>::Config as GenericConfig<D>>::Hasher:
            AlgebraicHasher<<L as PlonkParameters<D>>::Field>;
}

impl CommitmentMapperVariable for BeaconValidatorVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> PoseidonHashOutVariable
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let pubkey_hash = builder.poseidon_hash(&self.pubkey.variables());
        let withdrawal_credentials_hash =
            builder.poseidon_hash(&self.withdrawal_credentials.variables());

        let effective_balance_hash = builder.poseidon_hash(&self.effective_balance.variables());
        let slashed_hash = builder.poseidon_hash(&self.slashed.variables());

        let activation_eligibility_epoch_hash =
            builder.poseidon_hash(&self.activation_eligibility_epoch.variables());
        let activation_epoch_hash = builder.poseidon_hash(&self.activation_epoch.variables());

        let exit_epoch_hash = builder.poseidon_hash(&self.exit_epoch.variables());
        let withdrawable_epoch_hash = builder.poseidon_hash(&self.withdrawable_epoch.variables());

        let a1 = builder.poseidon_hash_pair(pubkey_hash, withdrawal_credentials_hash);
        let a2 = builder.poseidon_hash_pair(effective_balance_hash, slashed_hash);
        let a3 =
            builder.poseidon_hash_pair(activation_eligibility_epoch_hash, activation_epoch_hash);
        let a4 = builder.poseidon_hash_pair(exit_epoch_hash, withdrawable_epoch_hash);

        let b1 = builder.poseidon_hash_pair(a1, a2);
        let b2 = builder.poseidon_hash_pair(a3, a4);

        let zero_validator_pubkey = builder.constant::<BLSPubkeyVariable>([0; 48]);

        let is_zero_validator = builder.is_equal(self.pubkey, zero_validator_pubkey);

        let c = builder.poseidon_hash_pair(b1, b2);

        let zero = builder.zero::<Variable>();

        let zero_hash = PoseidonHashOutVariable {
            elements: ArrayVariable::new(vec![zero, zero, zero, zero]),
        };

        builder.select(is_zero_validator, zero_hash, c)
    }
}

pub fn poseidon_hash_tree_root_leafs<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    leafs: &[PoseidonHashOutVariable],
) -> PoseidonHashOutVariable
where
    <<L as PlonkParameters<D>>::Config as GenericConfig<D>>::Hasher:
        AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let mut leafs = leafs.to_vec();

    while leafs.len() != 1 {
        let mut tmp = Vec::new();

        for i in 0..leafs.len() / 2 {
            tmp.push(builder.poseidon_hash_pair(leafs[i * 2].clone(), leafs[i * 2 + 1].clone()));
        }

        leafs = tmp;
    }

    leafs[0].clone()
}
