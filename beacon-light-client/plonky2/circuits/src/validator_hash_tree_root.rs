use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_sha256::circuit::{array_to_bits, make_circuits, Sha256Targets};

pub struct Validator {
    pubkey: [bool; 256],
    withdrawal_credentials: [bool; 256],
    effective_balance: [bool; 256],
    slashed: [bool; 256],
    activation_eligibility_epoch: [bool; 256],
    activation_epoch: [bool; 256],
    exit_epoch: [bool; 256],
    withdrawable_epoch: [bool; 256],
}


pub fn hash_tree_root_validator_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pw: &mut PartialWitness<F>,
    validator: &Validator,
) {
    let hasher = make_circuits(builder, 512);

    for i in 0..256 {
        pw.set_bool_target(hasher.message[i], validator.pubkey[i]);
    }

    for i in 0..256 {
        pw.set_bool_target(hasher.message[i + 256], validator.withdrawal_credentials[i]);
    }


}
