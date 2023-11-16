use casper_finality_proofs::{
    combine_finality_votes::{
        concat_bitmasks::ConcatBitmasks,
        verify_subcommittee_vote::{
            VerifySubcommitteeVote, BITMASK_SIZE, BITMASK_SPLITS_COUNT, VALIDATORS_PER_COMMITTEE,
            VALIDATOR_SIZE_UPPER_BOUND, VARIABLES_COUNT_LITTLE_BITMASK,
        },
        CombineFinalityVotes,
    },
    verify_attestation_data::verify_split_bitmask::ValidatorBitmask,
};
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{ArrayVariable, CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};
use plonky2x::{backend::circuit::CircuitBuild, prelude::Field};
use rand::Rng;

/*
fn construct_upper_level_circuit<L: PlonkParameters<D>, const D: usize>(
    lower_level_circuit: &CircuitBuild<L, D>,
) -> CircuitBuild<L, D>
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let mut builder = CircuitBuilder::<L, D>::new();
    ConcatBitmasks::define(&mut builder, lower_level_circuit);
    builder.build()
}
*/

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    // let mut verify_subcomittee_vote_builder = CircuitBuilder::<L, D>::new();
    // VerifySubcommitteeVote::define(&mut verify_subcomittee_vote_builder);
    // let verify_subcommittee_vote = verify_subcomittee_vote_builder.build();

    let mut validator_bitmasks_builder = CircuitBuilder::<L, D>::new();
    ValidatorBitmask::define(&mut validator_bitmasks_builder);
    let validator_bitmasks = validator_bitmasks_builder.build();

    let rng = rand::thread_rng();
    let mut proofs = vec![];

    const LEVELS_TO_BUILD: usize = 1;
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
    /*
    for i in 0..LEVELS_TO_BUILD {
        let mut builder = CircuitBuilder::<L, D>::new();
        ConcatBitmasks::<i>::define(builder)
    }
    */
    // ConcatBitmasks<0>::define(builder, ...);

    for i in 0..BITMASK_SPLITS_COUNT {
        /*
        let random_set_bit: usize = rng.gen::<usize>() % BITMASK_SIZE;
        let mut input = verify_subcommittee_vote.input();
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(
            random_set_bit,
        ));
        */

        let range = rand::distributions::Uniform::new(0, VALIDATOR_SIZE_UPPER_BOUND as u64);
        let indices: Vec<<L as PlonkParameters<D>>::Field> = rng
            .clone()
            .sample_iter(&range)
            .map(|num| <L as PlonkParameters<D>>::Field::from_canonical_u64(num))
            .take(VALIDATORS_PER_COMMITTEE)
            .collect();

        let mut input = validator_bitmasks.input();
        input.write::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>(indices);
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(
            i * VARIABLES_COUNT_LITTLE_BITMASK,
        ));

        let (proof, _) = validator_bitmasks.prove(&input);
        proofs.push(proof);
    }

    // let mut inner_circuit = construct_upper_level_circuit(&validator_bitmasks);

    let mut level = 0;
    loop {
        println!("Proving {}th layer", level + 1);

        let inner_circuit = &circuits[level];

        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("{}th pair", i / 2 + 1);
            let mut input = inner_circuit.input();
            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());
            let (proof, _) = inner_circuit.prove(&input);
            new_proofs.push(proof);
        }
        proofs = new_proofs;
        println!("proofs size: {}", proofs.len());
        level += 1;

        if proofs.len() == 1 {
            break;
        }

        // inner_circuit = construct_upper_level_circuit(&inner_circuit);
    }
}
