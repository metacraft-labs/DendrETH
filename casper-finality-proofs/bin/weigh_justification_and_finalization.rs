use casper_finality_proofs::weigh_justification_and_finalization::{
    checkpoint::{CheckpointValue, CheckpointVariable},
    justification_bits::{JustificationBitsValue, JustificationBitsVariable},
    WeighJustificationAndFinalization,
};
use ethers::types::H256;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        ArrayVariable, Bytes32Variable, CircuitBuilder, DefaultParameters, PlonkParameters,
        U64Variable,
    },
    utils::bytes32,
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<L, D>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    let circuit = builder.build();

    let mut input = circuit.input();

    let beacon_state_root =
        bytes32!("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

    let slot = 6953401;

    let slot_proof: [H256; 5] = [
        bytes32!("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a"),
        bytes32!("96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1"),
        bytes32!("ef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f"),
        bytes32!("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217291,
        root: bytes32!("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    let previous_justified_checkpoint_proof = [
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

    let current_justified_checkpoint_proof = [
        bytes32!("0x2b913be7c761bbb483a1321ff90ad13669cbc422c8e23eccf9eb0137c8c3cf48"),
        bytes32!("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let justification_bits = JustificationBitsValue::<<L as PlonkParameters<D>>::Field> {
        bits: vec![true, true, true, true],
    };

    let justification_bits_proof = [
        bytes32!("0x1fca1f5d922549df42d4b5ca272bd4d022a77d520a201d5f24739b93f580a4e0"),
        bytes32!("0x9f1e3e59c7a4606e788c4e546a573a07c6c2e66ebd245aba2ff966b27e8c2d4f"),
        bytes32!("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    let previous_epoch_start_slot_root_in_block_roots_proof = [
        bytes32!("0x73dea1035b1bd431ccd1eaa893ad5f4b8488e68d2ca90615e5be0d8f7ba5a650"),
        bytes32!("0x0f7c6aa59235e573a4cdfb9411d5e4eb6255571814906c5928c016626aa2ff0a"),
        bytes32!("0xf770f73c2e01ddf6c71765e327eebb7bab0ab13f4506c736dfd6556037c0e646"),
        bytes32!("0x036f0750c86fdc21edee72b6ac1b5f728eed354c99d3b6862adf60f72bc5dcbe"),
        bytes32!("0x9730c8f3978ea7a1797603b19514e74273898f2be969ca8c583f55d14cd08d03"),
        bytes32!("0x47b601e8c14026380bdd0f716a4188e9f50a86bc58f0c342ead2a075ba8e5bc0"),
        bytes32!("0x6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        bytes32!("0x82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        bytes32!("0x30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        bytes32!("0xc9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        bytes32!("0x606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        bytes32!("0x4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        bytes32!("0xf3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        bytes32!("0xc524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        bytes32!("0xe3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        bytes32!("0x844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        bytes32!("0x2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("0x71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let current_epoch_start_slot_root_in_block_roots_proof = [
        bytes32!("c798192e5a066fe1ff3fc632bccd30a1ff47dc4d36909725db43ca6b23a5a7ba"),
        bytes32!("3161f17c79044792fc7c965a3fcb105f595bf895a44a774b871fa3017f5a36cc"),
        bytes32!("e3dddf89fa44413c3d4cf1762d7500b169116125194d96e86257cb616949560f"),
        bytes32!("3bfbdebbb29b9e066e08350d74f66116b0221c7d2c98724288a8e02bc7f937ae"),
        bytes32!("f50adbe1bff113f5d5535eee3687ac3b554af1eb56f8c966e572f8db3a61add3"),
        bytes32!("1a973e9b4fc1f60aea6d1453fe3418805a71fd6043f27a1c32a28bfcb67dd0eb"),
        bytes32!("6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        bytes32!("82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        bytes32!("30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        bytes32!("c9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        bytes32!("606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        bytes32!("4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        bytes32!("f3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        bytes32!("c524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        bytes32!("e3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        bytes32!("844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        bytes32!("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        bytes32!("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    ];

    let previous_epoch_start_slot_root_in_block_roots =
        bytes32!("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1");
    let current_epoch_start_slot_root_in_block_roots =
        bytes32!("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c");

    let total_active_balance = 10;
    let previous_epoch_target_balance = 10;
    let current_epoch_target_balance = 20;

    let finalized_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 217291,
        root: bytes32!("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    let finalized_checkpoint_proof = [
        bytes32!("0x26803d08d4a1a3d223ed199292fa78e62ef586391213548388375f302acfdece"),
        bytes32!("0xf0af1bff0357d4eb3b97bd6f7310a71acaff5c1c1d9dde7f20295b2002feccaf"),
        bytes32!("0x43e892858dc13eaceecec6b690cf33b7b85218aa197eb1db33de6bea3d3374c2"),
        bytes32!("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        bytes32!("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    ];

    input.write::<Bytes32Variable>(beacon_state_root);
    input.write::<U64Variable>(slot);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(slot_proof.to_vec());
    input.write::<CheckpointVariable>(previous_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(previous_justified_checkpoint_proof.to_vec());
    input.write::<CheckpointVariable>(current_justified_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(current_justified_checkpoint_proof.to_vec());
    input.write::<JustificationBitsVariable>(justification_bits);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(justification_bits_proof.to_vec());
    input.write::<U64Variable>(total_active_balance);
    input.write::<U64Variable>(previous_epoch_target_balance);
    input.write::<U64Variable>(current_epoch_target_balance);
    input.write::<Bytes32Variable>(previous_epoch_start_slot_root_in_block_roots);
    input.write::<ArrayVariable<Bytes32Variable, 18>>(
        previous_epoch_start_slot_root_in_block_roots_proof.to_vec(),
    );
    input.write::<Bytes32Variable>(current_epoch_start_slot_root_in_block_roots);
    input.write::<ArrayVariable<Bytes32Variable, 18>>(
        current_epoch_start_slot_root_in_block_roots_proof.to_vec(),
    );
    input.write::<CheckpointVariable>(finalized_checkpoint);
    input.write::<ArrayVariable<Bytes32Variable, 5>>(finalized_checkpoint_proof.to_vec());

    let (proof, mut output) = circuit.prove(&input);
    circuit.verify(&proof, &input, &output);

    let new_previous_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_current_justified_checkpoint = output.read::<CheckpointVariable>();
    let new_finalized_checkpoint = output.read::<CheckpointVariable>();
    let new_justification_bits = output.read::<JustificationBitsVariable>();

    println!("outputs:");
    println!(
        "new_previous_justified_checkpoint: {:?}",
        new_previous_justified_checkpoint
    );
    println!(
        "new_current_justified_checkpoint: {:?}",
        new_current_justified_checkpoint
    );
    println!("new_finalized_checkpoint: {:?}", new_finalized_checkpoint);
    println!("new_justification_bits: {:?}", new_justification_bits);
}
