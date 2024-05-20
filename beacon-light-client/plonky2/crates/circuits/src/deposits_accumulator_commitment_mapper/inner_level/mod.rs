#[cfg(test)]
mod test {
    use circuit::{Circuit, CircuitInput, SetWitness};
    use plonky2::iop::witness::WitnessWrite;
    use plonky2::{field::goldilocks_field::GoldilocksField, iop::witness::PartialWitness};

    use crate::deposits_accumulator_commitment_mapper::first_level::DepositsCommitmentMapperFirstLevel;
    use crate::utils::bits_to_bytes;
    use crate::validators_commitment_mapper::inner_level::ValidatorsCommitmentMapperInnerLevel;

    #[test]
    fn test_deposit_hash_tree_root_inner_level() {
        let (targets, circuit) = DepositsCommitmentMapperFirstLevel::build(&());

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee",
                  "depositIndex": "830987",
                  "depositMessageRoot": "9600f1c137423bfb703c9373918cfc299d7e939d2b428ec3eaf4b266d3638ef9",
                  "signature": "af92ccc88c4b1eca2f7dffb7c9288c014b2dc358d4846037a71f22a7ebab387795fd88fd29ab6304e25021fae7d99e320b8f9cbf6a5809a9b61e6612a2c838cea8f90a2e90172f111d17c429215d61452ee341ab17915c415696531ff9a69fe8"
                },
                "isReal": true
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof1 = circuit.prove(pw).unwrap();

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "957882961f53250f9b2b0ca1ad5b5f4fc1a89c3a55cd2dbba3df9e851f06c93e9fe2e691971884a269d4e40f3d054604",
                  "depositIndex": "830988",
                  "depositMessageRoot": "091982a48d67ce59d74113196d4467cb52e4450ed0b58f2f5e2de6a17aebd373",
                  "signature": "90b6e69502e7cf0bf1f385d121d1f60bd57d83081fcf96e130a7abc2beac046e1e28ec360e5455d226602d6ddd2eb39c162f09bc9420271eb8199c891d7bb52984a09fe416807ff7ed0eddd5eeeb2c07b13b38afafad77ec4f109828c8a73587"
                },
                "isReal": true
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof2 = circuit.prove(pw).unwrap();

        let (inner_level_targets, inner_level_circuit) =
            ValidatorsCommitmentMapperInnerLevel::build(&circuit);

        let mut pw = PartialWitness::<GoldilocksField>::new();

        pw.set_proof_with_pis_target(&inner_level_targets.proof1, &proof1);
        pw.set_proof_with_pis_target(&inner_level_targets.proof2, &proof2);

        let proof = inner_level_circuit.prove(pw).unwrap();

        let public_inputs =
            DepositsCommitmentMapperFirstLevel::read_public_inputs(&proof.public_inputs);

        let hex = hex::encode(bits_to_bytes(
            public_inputs.sha256_hash_tree_root.as_slice(),
        ));

        assert_eq!(
            hex,
            "adc7de41145866f80ea36dcc53fa7bd83854f40ff646c36cb8233943bf5e8068"
        );
    }

    #[test]
    fn test_deposit_hash_tree_root_inner_level_right_is_not_real() {
        let (targets, circuit) = DepositsCommitmentMapperFirstLevel::build(&());

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee",
                  "depositIndex": "830987",
                  "depositMessageRoot": "9600f1c137423bfb703c9373918cfc299d7e939d2b428ec3eaf4b266d3638ef9",
                  "signature": "af92ccc88c4b1eca2f7dffb7c9288c014b2dc358d4846037a71f22a7ebab387795fd88fd29ab6304e25021fae7d99e320b8f9cbf6a5809a9b61e6612a2c838cea8f90a2e90172f111d17c429215d61452ee341ab17915c415696531ff9a69fe8"
                },
                "isReal": true
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof1 = circuit.prove(pw).unwrap();

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "957882961f53250f9b2b0ca1ad5b5f4fc1a89c3a55cd2dbba3df9e851f06c93e9fe2e691971884a269d4e40f3d054604",
                  "depositIndex": "830988",
                  "depositMessageRoot": "091982a48d67ce59d74113196d4467cb52e4450ed0b58f2f5e2de6a17aebd373",
                  "signature": "90b6e69502e7cf0bf1f385d121d1f60bd57d83081fcf96e130a7abc2beac046e1e28ec360e5455d226602d6ddd2eb39c162f09bc9420271eb8199c891d7bb52984a09fe416807ff7ed0eddd5eeeb2c07b13b38afafad77ec4f109828c8a73587"
                },
                "isReal": false
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof2 = circuit.prove(pw).unwrap();

        let (inner_level_targets, inner_level_circuit) =
            ValidatorsCommitmentMapperInnerLevel::build(&circuit);

        let mut pw = PartialWitness::<GoldilocksField>::new();

        pw.set_proof_with_pis_target(&inner_level_targets.proof1, &proof1);
        pw.set_proof_with_pis_target(&inner_level_targets.proof2, &proof2);

        let proof = inner_level_circuit.prove(pw).unwrap();

        let public_inputs =
            DepositsCommitmentMapperFirstLevel::read_public_inputs(&proof.public_inputs);

        let hex = hex::encode(bits_to_bytes(
            public_inputs.sha256_hash_tree_root.as_slice(),
        ));

        assert_eq!(
            hex,
            "e17348b2f2054542b7820316e0549c5b91eb6713a380ecb78c6b93d6be772382"
        );
    }
}
