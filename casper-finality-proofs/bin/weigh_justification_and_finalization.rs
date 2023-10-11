use ethers::types::U256;
use plonky2x::{prelude::{PlonkParameters, Variable, DefaultParameters, CircuitBuilder}, backend::circuit::Circuit};
use casper_finality_proofs::test::TestCircuit;
use serde_derive::{Deserialize, Serialize};
use curta::math::field::Field;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    pub a: U256,
    pub b: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Outputs {
    pub c: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestInput {
    pub inputs: Inputs,
    pub outputs: Outputs,
}

    
    // TODO implement testManager to dynamically load test cases based on different Input so that wrappers become redundant
    // struct Test1 {
    //     test_data: String,
    //     type Schema = TestInput,
    // }
    fn main(){
        type L = DefaultParameters;
        const D: usize = 2;
        let json_data: TestInput = read_fixture("./src/test.json");
        
        let mut builder = CircuitBuilder::<L, D>::new();
        TestCircuit::define(&mut builder);
        let circuit = builder.build();
        let mut input = circuit.input();
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(json_data.inputs.a.as_u64()));
        input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(json_data.inputs.b.as_u64()));

        let (proof, mut output) = circuit.prove(&input);
        circuit.verify(&proof, &input, &output);
        let sum = output.read::<Variable>();
        assert_eq!(sum, <L as PlonkParameters<D>>::Field::from_canonical_u64(json_data.outputs.c.as_u64()));
    }

    #[allow(dead_code)] // We allow dead_code since this is used in tests
    fn read_fixture(filename: &str) -> TestInput {
        let mut file = File::open(filename).unwrap();
        let mut context = String::new();
        file.read_to_string(&mut context).unwrap();
    
        let context: TestInput = serde_json::from_str(context.as_str()).unwrap();
        context
    }
