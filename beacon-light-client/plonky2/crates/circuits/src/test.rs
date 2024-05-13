use circuit::public_inputs::field_reader::PublicInputsFieldReader;
use circuit::public_inputs::target_reader::PublicInputsTargetReader;
use circuit::set_witness::SetWitness;
use circuit::target_primitive::TargetPrimitive;
use circuit::to_targets::ToTargets;
use circuit::Circuit;
use circuit::TargetsWithPublicInputs;
use circuit_proc_macros::CircuitTarget;
use itertools::Itertools;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::HashOutTarget;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use serde::Deserialize;
use serde::Serialize;

use crate::utils::hashing::validator_hash_tree_root_poseidon::hash_tree_root_validator_poseidon;
use crate::utils::hashing::validator_hash_tree_root_poseidon::ValidatorTarget;

#[derive(CircuitTarget)]
pub struct TestTarget {
    #[target(in)]
    pub number: Target,

    #[target(in)]
    pub number2: Target,

    #[target(in)]
    pub numbers: [Target; 3],

    #[target(in)]
    pub validators: [ValidatorTarget; 2],

    #[target(out)]
    pub result: Target,

    #[target(out)]
    pub result2: Target,

    #[target(out)]
    pub validator_root_1: HashOutTarget,

    #[target(out)]
    pub validator_root_2: HashOutTarget,
}

pub struct TestCircuit {}

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

impl Circuit for TestCircuit {
    type F = F;
    type C = C;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Targets = TestTarget;
    type Params = ();

    fn define(builder: &mut CircuitBuilder<F, D>, _params: ()) -> Self::Targets {
        let number = builder.add_virtual_target();
        let number2 = builder.add_virtual_target();
        let result = builder.add(number, number2);

        let numbers = [(); 3].map(|_| builder.add_virtual_target());
        let mut result2 = builder.add(numbers[0], numbers[1]);
        result2 = builder.add(result2, numbers[2]);

        let validators = [(); 2].map(|_| hash_tree_root_validator_poseidon(builder));

        Self::Targets {
            number,
            number2,
            numbers,
            validators: validators
                .iter()
                .map(|validator_hash_tree_root| validator_hash_tree_root.validator.clone())
                .collect_vec()
                .try_into()
                .unwrap(),
            result,
            result2,
            validator_root_1: validators[0].hash_tree_root,
            validator_root_2: validators[1].hash_tree_root,
        }
    }
}

#[cfg(test)]
mod test {
    use circuit::array::Array;
    use circuit::{set_witness::SetWitness, Circuit, CircuitInput};
    use num::BigUint;
    use plonky2::iop::witness::PartialWitness;

    use crate::utils::hashing::validator_hash_tree_root_poseidon::ValidatorTargetPrimitive;

    use super::TestCircuit;

    #[test]
    fn some_test() {
        let (target, data) = TestCircuit::build(());

        let withdrawal_credentials_input: Array<bool, 256> =
            Array(vec![false; 256].try_into().unwrap());

        let validator_target_input = ValidatorTargetPrimitive {
            pubkey: Array(vec![false; 384].try_into().unwrap()),
            withdrawal_credentials: withdrawal_credentials_input.clone(),
            effective_balance: BigUint::from(1u64),
            slashed: true,
            activation_epoch: BigUint::from(2u64),
            withdrawable_epoch: BigUint::from(3u64),
            activation_eligibility_epoch: BigUint::from(4u64),
            exit_epoch: BigUint::from(5u64),
        };

        let input = CircuitInput::<TestCircuit> {
            number: 10,
            number2: 2,
            numbers: Array([1, 2, 3]),
            validators: Array([(); 2].map(|_| validator_target_input.clone())),
        };

        let mut pw = PartialWitness::new();
        target.set_witness(&mut pw, &input);
        let proof = data.prove(pw).unwrap();
        let public_inputs = TestCircuit::read_public_inputs(&proof.public_inputs);
        println!("result: {:?}", public_inputs);
    }
}
