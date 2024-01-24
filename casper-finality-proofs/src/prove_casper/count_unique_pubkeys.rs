use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::{backend::circuit::CircuitBuild, frontend::{builder::CircuitBuilder, uint::uint64::U64Variable}};
use plonky2x::prelude::PlonkParameters;
use crate::{combine_finality_votes::count_unique_validators::CountUniqueValidators, constants::{TEST_ATTESTATIONS_READ, TEST_VALIDATORS_IN_COMMITMENT_SIZE, VALIDATOR_INDICES_IN_SPLIT}, utils::json::read_json_from_file};

fn count_unique_validators_in_chunk<L: PlonkParameters<D>, const D: usize>(
    circuit: &CircuitBuild<L,D>,
    chunk: &[u64],
    sigma: u64
) -> ProofWithPublicInputs<<L as PlonkParameters<D>>::Field, <L as PlonkParameters<D>>::Config, D>
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{

    let mut input = circuit.input();
    input.write::<U64Variable>(sigma);
    for validator_index in chunk {
        input.write::<U64Variable>(validator_index.clone());
    }
    println!("CHUNK: {:?}", chunk);
    let (proof, mut _output) = circuit.prove(&input);
    println!("Output: {:?}", _output);

    proof
}

pub fn count_unique_validators<L: PlonkParameters<D>, const D: usize>(
    file_path_sorted_validators: &str,
) -> (
    Vec<ProofWithPublicInputs<<L as PlonkParameters<D>>::Field, <L as PlonkParameters<D>>::Config, D>>,
    CircuitBuild<L,D>
)
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let sorted_validators_json = read_json_from_file(file_path_sorted_validators).unwrap();

    let mut cuv_builder = CircuitBuilder::<L, D>::new();
    CountUniqueValidators::define(&mut cuv_builder);

    let cuv_circuit = cuv_builder.build();

    let sorted_validators: Vec<u64> = sorted_validators_json.as_array()
        .unwrap()
        .iter()
        .take(TEST_VALIDATORS_IN_COMMITMENT_SIZE * TEST_ATTESTATIONS_READ) //TODO: This is Test Size
        .map(|validator| serde_json::from_value(validator.clone()).unwrap())
        .collect();

    let chunk_size = VALIDATOR_INDICES_IN_SPLIT;
    let chunked_iter = sorted_validators.chunks_exact(chunk_size);
    //TODO: Use chunked_iter.into_remainder to parse final slice of validators

    let mut count_unique_validators_proofs = vec![];
    let mut counter = 0;
    let sigma: u64 = 1;
    for chunk in chunked_iter { 
        println!("===Proving Chunk {}====",counter);
        counter += 1;
        let chunk_proof = count_unique_validators_in_chunk(&cuv_circuit, chunk, sigma);

        count_unique_validators_proofs.push(chunk_proof);
    }

    (count_unique_validators_proofs, cuv_circuit)
}
