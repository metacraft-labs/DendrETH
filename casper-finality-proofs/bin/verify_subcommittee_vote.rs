use plonky2x::{
    backend::circuit::{Circuit, PublicOutput},
    prelude::{
        ArrayVariable, CircuitBuilder, 
        DefaultParameters, PlonkParameters,
        Field, Variable, BoolVariable
    },
};
use rand::Rng;

use casper_finality_proofs::verify_attestation_data::verify_split_bitmask::ValidatorBitmask;
use casper_finality_proofs::combine_finality_votes::verify_subcommittee_vote::{
    VALIDATORS_PER_COMMITTEE,
    VALIDATOR_SIZE_UPPER_BOUND,
    VARIABLES_COUNT_LITTLE_BITMASK,
    };

fn main() {
    plonky2x::utils::setup_logger();

    type L = DefaultParameters;
    const D: usize = 2;

    let mut builder = CircuitBuilder::<L, D>::new();
    ValidatorBitmask::define::<L, D>(&mut builder); 
        
    let circuit = builder.build();

    let mut input = circuit.input();

    let mut rng = rand::thread_rng();
    let range = rand::distributions::Uniform::new(0, VALIDATOR_SIZE_UPPER_BOUND as u64);
    let values: Vec<<L as PlonkParameters<D>>::Field> = 
        rand::thread_rng()
        .sample_iter(&range)
        .map(|num| <L as PlonkParameters<D>>::Field::from_canonical_u64(num))
        .take(VALIDATORS_PER_COMMITTEE)
        .collect();

    
    println!("values: {:?}\n", values);
    // let vec_filtered = Vec<L as PlonkParameters<D>>::Field>::new();


    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(0)); // Begin Range
    input.write::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>(values);

    // let mut output: Option<PublicOutput<L, D>> = None;
    let (proof, mut output) = circuit.prove(&input);

    let _target = output.read::<Variable>();
    let _source = output.read::<Variable>();
    let _bls_signature = output.read::<Variable>();
    let _voted_count = output.read::<Variable>();
    let _range_begin = output.read::<Variable>();

    let bitmask = output.read::<ArrayVariable<Variable, VARIABLES_COUNT_LITTLE_BITMASK>>();
    println!("Bitmask: {:?}", bitmask );

}
