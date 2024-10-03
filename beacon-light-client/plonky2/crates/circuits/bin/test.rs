use circuit::serde::serde_u128_str;
use std::str::FromStr;

use circuit::{
    serde::serde_u64_str,
    targets::uint::{
        ops::{
            arithmetic::{Add, Div, Mul, Rem, Sub},
            comparison::{Comparison, EqualTo, LessThanOrEqual},
        },
        Uint128Target, Uint512Target, Uint64Target,
    },
    Circuit, SSZHashTreeRoot, SetWitness,
};
use circuit_derive::CircuitTarget;
use circuits::{
    common_targets::SSZTarget,
    serializers::{biguint_to_str, parse_biguint},
    utils::circuit::verify_proof,
};
use num::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};
use serde_json::json;

#[derive(CircuitTarget)]
struct TestCircuitTarget {
    #[target(in)]
    #[serde(with = "serde_u64_str")]
    a: Uint64Target,

    #[target(in)]
    #[serde(with = "serde_u64_str")]
    b: Uint64Target,

    #[target(out)]
    res: Uint64Target,

    #[target(out)]
    cmp: BoolTarget,

    #[target(out)]
    a_le_bits: [BoolTarget; 64],

    #[target(out)]
    a_le_bytes: [BoolTarget; 64],

    #[target(out)]
    #[serde(with = "serde_u64_str")]
    truncated: Uint64Target,
    // #[target(out)]
    // ssz: SSZTarget,
}

struct TestCircuit;

impl Circuit for TestCircuit {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = TestCircuitTarget;

    fn define(builder: &mut CircuitBuilder<Self::F, 2>, _: &Self::Params) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        // let a_le_bits = input.a.to_le_bits(builder);
        // let a_reconstructed = Uint64Target::from_le_bits(&a_bits, builder);
        let biguint = builder.constant_biguint(&BigUint::from_str("10").unwrap());

        let truncated = Uint64Target::truncate_biguint(&biguint, builder);

        let a_le_bytes = input.a.to_le_bytes(builder);
        let a_reconstructed = Uint64Target::from_le_bytes(&a_le_bytes, builder);
        let a_le_bits = a_reconstructed.to_le_bits(builder);

        // let first = IntegerTarget::constant(builder, U512::from_dec_str("253829385737293451256943384527124575849328384758392012847495847338472938475849383743829374839238473").unwrap());
        // let second = IntegerTarget::constant(
        //     builder,
        //     U512::from_dec_str("8346729346034815620398561230958612309847").unwrap(),
        // );
        // let res = first.sub(second, builder);
        let res = a_reconstructed;

        // let ssz = input.a.ssz_hash_tree_root(builder);

        Self::Target {
            a: input.a,
            b: input.b,
            res,
            cmp: input.a.lt(input.b, builder),
            a_le_bits: a_le_bits.clone().try_into().unwrap(),
            a_le_bytes: a_le_bytes.try_into().unwrap(),
            truncated, // ssz,
        }
    }
}

#[derive(CircuitTarget)]
struct RecursiveTestCircuitTarget {
    proof: ProofWithPublicInputsTarget<2>,

    #[target(out)]
    res: Uint64Target,
}

struct RecursiveTestCircuit;

impl Circuit for RecursiveTestCircuit {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = RecursiveTestCircuitTarget;

    type Params = CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, 2>,
        circuit_data: &Self::Params,
    ) -> Self::Target {
        let proof = verify_proof(builder, circuit_data);
        let public_inputs = TestCircuit::read_public_inputs_target(&proof.public_inputs);

        let res = public_inputs.res;
        // let b = IntegerTarget::constant(builder, 100);

        // res = res.sub(b, builder);

        Self::Target { proof, res }
    }
}

pub fn main() {
    type F = GoldilocksField;

    let (target, data) = TestCircuit::build(&());

    let mut pw = PartialWitness::<F>::new();
    target.set_witness(
        &mut pw,
        &serde_json::from_str(
            &json!({
                "a": "290910267100917",
                "b": "200"
            })
            .to_string(),
        )
        .unwrap(),
    );

    let proof = data.prove(pw).unwrap();

    let output = TestCircuit::read_public_inputs(&proof.public_inputs);
    println!("output: {:?}", output);

    let (recursive_target, recursive_data) = RecursiveTestCircuit::build(&data);
    let mut pw = PartialWitness::<F>::new();
    pw.set_proof_with_pis_target(&recursive_target.proof, &proof);
    let recursive_proof = recursive_data.prove(pw).unwrap();

    let recursive_output = RecursiveTestCircuit::read_public_inputs(&recursive_proof.public_inputs);
    println!("recursive output: {:?}", recursive_output);
}
