use casper_finality_proofs::combine_finality_votes::{
    verify_subcommittee_vote::{VerifySubcommitteeVote, BITMASK_SIZE},
    CombineFinalityVotes,
};
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};
use plonky2x::{backend::circuit::CircuitBuild, prelude::Field};
use rand::Rng;

fn construct_upper_level_circuit<L: PlonkParameters<D>, const D: usize>(
    lower_level_circuit: &CircuitBuild<L, D>,
) -> CircuitBuild<L, D>
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let mut builder = CircuitBuilder::<L, D>::new();
    CombineFinalityVotes::define(&mut builder, lower_level_circuit);
    builder.build()
}

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let mut verify_subcomittee_vote_builder = CircuitBuilder::<L, D>::new();
    VerifySubcommitteeVote::define(&mut verify_subcomittee_vote_builder);
    let verify_subcommittee_vote = verify_subcomittee_vote_builder.build();

    let mut rng = rand::thread_rng();

    let mut proofs = vec![];
    for _ in 0..2usize.pow(1) {
        let random_set_bit: usize = rng.gen::<usize>() % BITMASK_SIZE;
        let mut input = verify_subcommittee_vote.input();
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(
            random_set_bit,
        ));

        let (proof, _) = verify_subcommittee_vote.prove(&input);
        proofs.push(proof);
    }

    let mut inner_circuit = construct_upper_level_circuit(&verify_subcommittee_vote);

    let mut level = 1;
    loop {
        println!("Proving {}th layer", level);
        level += 1;

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

        if proofs.len() == 1 {
            break;
        }

        inner_circuit = construct_upper_level_circuit(&inner_circuit);
    }
}
