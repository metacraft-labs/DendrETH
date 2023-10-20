use std::println;

use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        eth::vars::BLSPubkeyVariable,
        vars::{EvmVariable, SSZVariable},
    },
    prelude::{
        BoolVariable, ByteVariable, Bytes32Variable, CircuitVariable, U64Variable, Variable,
    },
    utils::bytes32,
};

#[derive(Debug, Copy, Clone, CircuitVariable)]
#[value_name(ValidatorValue)]
pub struct ValidatorVariable {
    pub pubkey: BLSPubkeyVariable,
    pub withdrawal_credentials: Bytes32Variable,
    pub effective_balance: U64Variable,
    pub slashed: BoolVariable,
    pub activation_eligibility_epoch: U64Variable,
    pub activation_epoch: U64Variable,
    pub exit_epoch: U64Variable,
    pub withdrawable_epoch: U64Variable,
}

impl SSZVariable for ValidatorVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let zero = builder.constant::<ByteVariable>(0);
        let one = builder.constant::<ByteVariable>(1);

        let mut pubkey_serialized = self.pubkey.0 .0.to_vec();
        pubkey_serialized.extend([zero; 16]);

        let tmp = builder.curta_sha256(&pubkey_serialized);
        builder.watch(&tmp, "pubkey_hash");
        let mut a1 = tmp.0 .0.to_vec();
        a1.extend(self.withdrawal_credentials.0 .0.to_vec());

        let mut a2 = self.effective_balance.encode(builder);
        a2.reverse();
        a2.extend([zero; 24]);
        let mut slashed = vec![builder.select(self.slashed, one, zero)];
        slashed.extend([zero; 31]);
        a2.extend(slashed);

        let mut a3 = self.activation_eligibility_epoch.encode(builder);
        a3.reverse();
        a3.extend([zero; 24]);
        let mut tmp = self.activation_epoch.encode(builder);
        tmp.reverse();
        tmp.extend([zero; 24]);
        a3.extend(&tmp);

        let mut a4 = self.exit_epoch.encode(builder);
        a4.reverse();
        a4.extend([zero; 24]);
        let mut tmp = self.withdrawable_epoch.encode(builder);
        tmp.reverse();
        tmp.extend([zero; 24]);
        a4.extend(&tmp);

        let mut b1 = builder.curta_sha256(&a1).0 .0.to_vec();
        b1.extend(builder.curta_sha256(&a2).0 .0.to_vec());

        let mut b2 = builder.curta_sha256(&a3).0 .0.to_vec();
        b2.extend(builder.curta_sha256(&a4).0 .0.to_vec());

        let mut c1 = builder.curta_sha256(&b1).0 .0.to_vec();
        c1.extend(builder.curta_sha256(&b2).0 .0.to_vec());

        let leaf = builder.curta_sha256(&c1);
        let zero_leaf = builder.constant::<Bytes32Variable>(bytes32!(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        ));
        let zero_validator_pubkey = builder.constant::<BLSPubkeyVariable>([0; 48]);
        let is_zero_validator = builder.is_equal(self.pubkey, zero_validator_pubkey);
        builder.select(is_zero_validator, zero_leaf, leaf)
    }
}
