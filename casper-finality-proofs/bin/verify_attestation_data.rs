use curta::chip::field;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable}, frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable}, vars::{Bytes32Variable, U256Variable}, uint::uint64::U64Variable},
};
use casper_finality_proofs::verify_attestation_data::verify_attestation_data::{VerifyAttestationData};
fn main() {
    plonky2x::utils::setup_logger();

    type L = DefaultParameters;
    const D: usize = 2;

    let mut builder = CircuitBuilder::<L, D>::new();
        
    VerifyAttestationData::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    // Json with

    // input.write::<ArrayVariable<Variable, VALIDATOR_NUM>>(values_vec);

    // let mut output: Option<PublicOutput<L, D>> = None;
    // let (proof, mut output) = circuit.prove(&input);

    // let result = output.read::<Variable>();
    // println!("Bitmask: {:?}", result );

}
