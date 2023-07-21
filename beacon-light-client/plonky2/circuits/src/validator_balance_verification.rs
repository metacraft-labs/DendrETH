use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_sha256::circuit::{make_circuits, Sha256Targets};

pub struct ValidatorBalanceVerificationTargets {
    pub rangeStart: u64,
    pub rangeEnd: u64,
    pub rangeTotalValue: u64,
    pub rangeDepositsCount: u64,
    pub balancesRoot: [BoolTarget; 256],
    pub validatorsAccumulatorCommitment: HashOutTarget,
    pub validatorCommitment: HashOutTarget,
    pub pubkeys: [[Target; 7]; 32],
    pub activationEpochs: [[Target; 2]; 32],
    pub exitEpochs: [[Target; 2]; 32],
    pub balances: [[BoolTarget; 256]; 8],
}

pub fn validator_balance_verification<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves_len: usize,
) -> HashTreeRootTargets {
    let leaves: Vec<[BoolTarget; 256]> = (0..leaves_len)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let mut hashers: Vec<Sha256Targets> = Vec::new();

    for i in 0..(leaves_len / 2) {
        hashers.push(make_circuits(builder, 512));

        for j in 0..256 {
            builder.connect(hashers[i].message[j].target, leaves[i * 2][j].target);
            builder.connect(
                hashers[i].message[j + 256].target,
                leaves[i * 2 + 1][j].target,
            );
        }
    }

    let mut k = 0;
    for i in leaves_len / 2..leaves_len - 1 {
        hashers.push(make_circuits(builder, 512));

        for j in 0..256 {
            builder.connect(
                hashers[i].message[j].target,
                hashers[k * 2].digest[j].target,
            );
            builder.connect(
                hashers[i].message[j + 256].target,
                hashers[k * 2 + 1].digest[j].target,
            );
        }

        k += 1;
    }

    HashTreeRootTargets {
        leaves: leaves,
        hash_tree_root: hashers[leaves_len - 2].digest.clone().try_into().unwrap(),
    }
}
