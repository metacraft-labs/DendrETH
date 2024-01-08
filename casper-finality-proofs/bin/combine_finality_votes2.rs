use casper_finality_proofs::{
    combine_finality_votes::{
        circuit2::{
            CommitTrustedValidatorPubkeys, Commitment, PublicKey,
        },
        commitment_accumulator_inner::CommitmentAccumulatorInner,
    },
    verify_attestation_data::verify_attestation_data::VerifyAttestationData,
    weigh_justification_and_finalization::checkpoint::{CheckpointValue, CheckpointVariable},
};
use itertools::{izip, multiunzip};
use plonky2x::{
    backend::circuit::{Circuit, PublicOutput},
    prelude::{ArrayVariable, CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};
use plonky2x::{
    prelude::{BoolVariable, Field},
    utils::bytes32,
};
use primitive_types::U256;
use casper_finality_proofs::constants::{VALIDATORS_PER_COMMITTEE};

struct Subcommittee {
    pub pubkeys: Vec<U256>,
    pub indices: Vec<u64>,
    pub trusted_bitmask: Vec<bool>,
    pub count: u64,
}

impl Subcommittee {
    pub fn normalize(&mut self) {
        self.pubkeys
            .resize_with(VALIDATORS_PER_COMMITTEE, Default::default);
        self.indices
            .resize_with(VALIDATORS_PER_COMMITTEE, Default::default);
        self.trusted_bitmask
            .resize_with(VALIDATORS_PER_COMMITTEE, Default::default);
    }
}

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let mut subcommittees = [ 
        Subcommittee {
            pubkeys: vec![U256::from(1), U256::from(2), U256::from(3)],
            indices: vec![20, 10, 15],
            trusted_bitmask: vec![true, false, true],
            count: 3,
        },
        Subcommittee {
            pubkeys: vec![U256::from(4), U256::from(5), U256::from(6), U256::from(7)],
            indices: vec![100, 420, 69, 101],
            trusted_bitmask: vec![false, true, true, false],
            count: 4,
        },
        Subcommittee {
            pubkeys: vec![U256::from(8), U256::from(9), U256::from(10), U256::from(11)],
            indices: vec![1, 5, 104, 3],
            trusted_bitmask: vec![true, true, true, true],
            count: 4,
        },
        Subcommittee {
            pubkeys: vec![U256::from(12)],
            indices: vec![7],
            trusted_bitmask: vec![false],
            count: 1,
        },
    ];

    let mut result = U256::from(0);
    for subcom in &subcommittees {
        for i in 0..subcom.pubkeys.len() {
            if subcom.trusted_bitmask[i] {
                result += subcom.pubkeys[i] * U256::from(10);
                print!("{}, ", subcom.pubkeys[i]);
            }
        }
    }

    println!("\nresult = {}", result);

    let mut builder = CircuitBuilder::<L, D>::new();
    VerifyAttestationData::define(&mut builder);
    let leaf_circuit = builder.build();
    // let input = leaf_circuit.input();

    let source_checkpoint = CheckpointValue {
        epoch: 2,
        root: bytes32!("0x0000000000000000000000000000000000000000000000000000000000000002"),
    };

    let target_checkpoint = CheckpointValue {
        epoch: 3,
        root: bytes32!("0x0000000000000000000000000000000000000000000000000000000000000003"),
    };

    /*
    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(4));

    input.write::<CheckpointVariable>(source_checkpoint);
    input.write::<CheckpointVariable>(target_checkpoint);

    input.write::<ArrayVariable<PublicKey, VALIDATORS_PER_SUBCOMMITTEE>>(pubkeys_vec.clone());
    input.write::<ArrayVariable<BoolVariable, VALIDATORS_PER_SUBCOMMITTEE>>(
        trusted_validators_bitmask_vec.clone(),
    );

    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(3));
    */

    // let (proof, mut output) = circuit.prove(&input);
    // circuit.verify(&proof, &input, &output);

    // let commitment = output.read::<U256Variable>();
    // let target = output.read::<CheckpointVariable>();
    // let source = output.read::<CheckpointVariable>();

    let random_value = <L as PlonkParameters<D>>::Field::from_canonical_usize(10);

    let mut proofs = vec![];

    for i in 0..subcommittees.len() {
        let mut input = leaf_circuit.input();

        subcommittees[i].normalize();

        input.write::<Variable>(random_value);
        input.write::<CheckpointVariable>(source_checkpoint.clone());
        input.write::<CheckpointVariable>(target_checkpoint.clone());
        input.write::<ArrayVariable<PublicKey, VALIDATORS_PER_COMMITTEE>>(
            subcommittees[i].pubkeys.clone(),
        );
        input.write::<ArrayVariable<BoolVariable, VALIDATORS_PER_COMMITTEE>>(
            subcommittees[i].trusted_bitmask.clone(),
        );
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
            subcommittees[i].count,
        ));

        let (proof, _) = leaf_circuit.prove(&input);
        proofs.push(proof);
    }

    let mut child_circuit = leaf_circuit;

    let mut level = 0;
    loop {
        println!("Proving {}th layer", level + 1);

        let mut builder = CircuitBuilder::<L, D>::new();
        CommitmentAccumulatorInner::define(&mut builder, &child_circuit);
        let inner_circuit = builder.build();
        // let mut input = leaf_circuit.input();

        let mut final_output: Option<PublicOutput<L, D>> = None;

        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("{}th pair", i / 2 + 1);
            let mut input = inner_circuit.input();
            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());
            let (proof, output) = inner_circuit.prove(&input);
            final_output = Some(output);
            new_proofs.push(proof);
        }
        proofs = new_proofs;
        println!("proofs size: {}", proofs.len());
        level += 1;

        if proofs.len() == 1 {
            let mut final_output = final_output.unwrap();
            let commitment = final_output.proof_read::<Commitment>();
            let _target = final_output.proof_read::<CheckpointVariable>();
            let _source = final_output.proof_read::<CheckpointVariable>();

            println!("commitment: {:?}", commitment);

            // println!("validators: {:?}", indices);
            // println!("packed bitmask: {:?}", bitmask);
            break;
        }
        child_circuit = inner_circuit;
    }

    let pubkeys = subcommittees.iter().fold(vec![], |mut acc, subcommittee| {
        let mut other = subcommittee.pubkeys[0..subcommittee.count as usize].to_vec();
        acc.append(&mut other);
        acc
    });

    let trusted_bitmask = subcommittees.iter().fold(vec![], |mut acc, subcommittee| {
        let mut other = subcommittee.trusted_bitmask[0..subcommittee.count as usize].to_vec();
        acc.append(&mut other);
        acc
    });

    let indices = subcommittees.iter().fold(vec![], |mut acc, subcommittee| {
        let mut other = subcommittee.indices[0..subcommittee.count as usize].to_vec();
        acc.append(&mut other);
        acc
    });

    let mut zipped = izip!(&indices, &pubkeys, &trusted_bitmask,).collect::<Vec<_>>();

    zipped.sort_by(|(idx1, _, _), (idx2, _, _)| idx1.cmp(idx2));

    let (indices, pubkeys, trusted_bitmask): (Vec<u64>, Vec<U256>, Vec<bool>) = multiunzip(zipped);

    println!("pubkeys: {:?}", pubkeys);
    println!("indices: {:?}", indices);
    println!("trusted_bitmask: {:?}", trusted_bitmask);
}
