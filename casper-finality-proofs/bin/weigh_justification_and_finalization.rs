use casper_finality_proofs::weigh_justification_and_finalization::{
    CheckpointValue, CheckpointVariable, WeighJustificationAndFinalization,
};
use ethers::types::H256;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        ArrayVariable, Bytes32Variable, CircuitBuilder, DefaultParameters, PlonkParameters,
        U64Variable,
    },
    utils::{bytes, bytes32},
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<L, D>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    let beacon_state_root: [u8; 32] =
        bytes!("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

    let slot_proof: [H256; 5] = [
        bytes32!("0xb85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a"),
        bytes32!("0x96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1"),
        bytes32!("0xef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f"),
        bytes32!("0x2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("0x71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217291,
        root: bytes32!("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    let previous_justified_checkpoint_proof: [H256; 5] = [
        bytes32!("0xf7b1fc5e9ef34f7455c8cc475a93eccc5cd05a3879e983a2bad46bbcbb2c71f5"),
        bytes32!("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let current_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217292,
        root: bytes32!("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1"),
    };

    let current_justified_checkpoint_proof: [H256; 5] = [
        bytes32!("0x2b913be7c761bbb483a1321ff90ad13669cbc422c8e23eccf9eb0137c8c3cf48"),
        bytes32!("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    input.write::<Bytes32Variable>(H256(beacon_state_root));
    input.write::<U64Variable>(6953401);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(slot_proof.to_vec());
    input.write::<CheckpointVariable>(previous_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(previous_justified_checkpoint_proof.to_vec());
    input.write::<CheckpointVariable>(current_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(current_justified_checkpoint_proof.to_vec());

    let (proof, output) = circuit.prove(&input);
    circuit.verify(&proof, &input, &output);
}
