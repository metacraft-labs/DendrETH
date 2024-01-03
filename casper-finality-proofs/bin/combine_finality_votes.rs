use casper_finality_proofs::{
    combine_finality_votes::{
        concat_bitmasks::ConcatBitmasks,
        verify_subcommittee_vote::{
            BITMASK_SPLITS_COUNT, PACK_SIZE, VALIDATORS_PER_COMMITTEE, VALIDATOR_SIZE_UPPER_BOUND,
            VARIABLES_COUNT_BIG_BITMASK, VARIABLES_COUNT_LITTLE_BITMASK
        },
    },
    verify_attestation_data::verify_split_bitmask_deprecated::ValidatorBitmask,
};
use plonky2x::prelude::Field;
use plonky2x::{
    backend::circuit::{Circuit, PublicOutput},
    prelude::{ArrayVariable, CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};
use rand::Rng;

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let mut validator_bitmasks_builder = CircuitBuilder::<L, D>::new();
    ValidatorBitmask::define(&mut validator_bitmasks_builder);
    let validator_bitmasks = validator_bitmasks_builder.build();

    let rng = rand::thread_rng();
    let mut proofs = vec![];

    // const LEVELS_TO_BUILD: usize = 1;
    let mut circuits = vec![];

    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<0>::define(&mut builder, &validator_bitmasks);
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<1>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<2>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<3>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<4>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<5>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<6>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<7>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<8>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<9>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<10>::define(&mut builder, &circuits.last().unwrap());
        circuits.push(builder.build());
    }
    


    let range = rand::distributions::Uniform::new(0, VALIDATOR_SIZE_UPPER_BOUND as u64);
    let indices: Vec<<L as PlonkParameters<D>>::Field> = rng
        .clone()
        .sample_iter(&range)
        .map(|num| <L as PlonkParameters<D>>::Field::from_canonical_u64(num))
        .take(VALIDATORS_PER_COMMITTEE)
        .collect();

    const PROOFS_COUNT: usize = 1024; // iztrii go tva
    for i in 0..PROOFS_COUNT {
        let mut input = validator_bitmasks.input();
        input.write::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>(indices.clone());
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(
            i * VARIABLES_COUNT_LITTLE_BITMASK * PACK_SIZE,
        ));

        let (proof, _) = validator_bitmasks.prove(&input);
        proofs.push(proof);
    }

    let mut level = 0;
    loop {
        println!("Proving {}th layer", level + 1);

        let inner_circuit = &circuits[level];
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
            let _target = final_output.proof_read::<Variable>();
            let _source = final_output.proof_read::<Variable>();
            let _bls_signature = final_output.proof_read::<Variable>();
            let _voted_count = final_output.proof_read::<Variable>();
            let _range_begin = final_output.proof_read::<Variable>();
            let bitmask =
                // final_output.proof_read::<ArrayVariable<Variable, VARIABLES_COUNT_BIG_BITMASK>>();
                final_output.proof_read::<ArrayVariable<Variable, {VARIABLES_COUNT_LITTLE_BITMASK * PROOFS_COUNT}>>();
            println!("validators: {:?}", indices);
            println!("packed bitmask: {:?}", bitmask);
            break;
        }
    }
}
