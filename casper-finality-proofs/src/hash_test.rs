use itertools::Itertools;
use plonky2x::{
    backend::circuit::Circuit,
    frontend::eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
    prelude::{
        Bytes32Variable, CircuitBuilder, CircuitVariable, Div, PlonkParameters, U64Variable,
    },
};

#[derive(Debug, Clone)]
pub struct HashTestCircuit;

impl Circuit for HashTestCircuit {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let a = builder.read::<Bytes32Variable>();
        let b = builder.read::<Bytes32Variable>();

        let slot = builder.read::<U64Variable>();

        let c = builder.sha256(
            a.0 .0
                .iter()
                .chain(b.0 .0.iter())
                .cloned()
                .collect_vec()
                .as_slice(),
        );

        let slots_per_epoch = U64Variable::constant(builder, 32);

        let epoch = slot.div(slots_per_epoch, builder);

        let mut validator = builder.read::<BeaconValidatorVariable>();

        validator.pubkey = BLSPubkeyVariable::constant(builder, [0; 48]);

        builder.write(c);
        builder.write(epoch);

        builder.write(validator);
    }
}
