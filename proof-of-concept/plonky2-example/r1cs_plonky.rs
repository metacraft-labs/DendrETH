use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;

use anyhow::Result;
use hex::ToHex;
use itertools::Itertools;
use num::bigint::Sign;
use num::{BigInt, BigUint};
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use r1cs_file::R1csFile;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

// #[derive(Debug, Default, PartialEq, Eq)]
// pub struct Constraint(
//     pub Vec<(F, usize)>,
//     pub Vec<(F, usize)>,
//     pub Vec<(F, usize)>,
// );

fn main() -> () {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    // The arithmetic circuit.

    println!("{}", current_dir().unwrap().to_str().unwrap());
    let f = File::open("circuit.r1cs").unwrap();
    let r: R1csFile<32> = R1csFile::read(f).unwrap();

    let mut witness_map: HashMap<u32, bool> = HashMap::new();
    for i in 0..399 {
        witness_map.insert(i, true);
    }

    for c in r.constraints.0 {
        // A
        let counter_a =
            c.0.iter()
                .filter(|x| !witness_map.contains_key(&x.1))
                .count();

        // B
        let counter_b =
            c.1.iter()
                .filter(|x| !witness_map.contains_key(&x.1))
                .count();

        // C
        let counter_c =
            c.2.iter()
                .filter(|x| !witness_map.contains_key(&x.1))
                .count();

        if counter_a > 0 {
            println!("Unknown A HERE");
            let p =
                c.0.iter()
                    .position(|x| !witness_map.contains_key(&x.1))
                    .unwrap();
            println!("{}", c.0[p].1);
            witness_map.insert(c.0[p].1, true);
        }

        if counter_b > 0 {
            println!("Unknown B HERE");
            let p =
                c.1.iter()
                    .position(|x| !witness_map.contains_key(&x.1))
                    .unwrap();
            println!("{}", c.1[p].1);
            witness_map.insert(c.1[p].1, true);
        }

        if counter_c > 0 {
            let p =
                c.2.iter()
                    .position(|x| !witness_map.contains_key(&x.1))
                    .unwrap();
            witness_map.insert(c.2[p].1, true);
        }

        if counter_a + counter_b + counter_c > 1 {
            panic!("OPA");
        }

        // let f = &c.0[0].0;
        // print!("{}", c.0[0].1);
        // let b = BigUint::from_bytes_le(f.as_bytes());
        // println!("{}", f.as_bytes().encode_hex::<String>());
        // println!("{}", b);
    }

    // let constraints = vec![
    //     Constraint(
    //         vec![(F::NEG_ONE, 2)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 3)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 3)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 4)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 4)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 5)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 5)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 6)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 6)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 7)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 7)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 8)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 8)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 9)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 9)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 10)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 10)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 11)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 11)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 12)],
    //     ),
    //     Constraint(
    //         vec![(F::NEG_ONE, 12)],
    //         vec![(F::from_canonical_u32(1), 2)],
    //         vec![(F::NEG_ONE, 1)],
    //     ),
    // ];

    // let mut witness_map: HashMap<usize, Target> = HashMap::new();

    // witness_map.insert(0, builder.add_virtual_target());
    // witness_map.insert(1, builder.add_virtual_target());
    // witness_map.insert(2, builder.add_virtual_target());

    // for constraint in constraints {
    //     let has_unknown_A = constraint.0.iter().any(|x| !witness_map.contains_key(&x.1));

    //     if has_unknown_A {
    //         panic!("A contains unknown");
    //     }

    //     let targets_a: Vec<Target> = constraint
    //         .0
    //         .iter()
    //         .map(|x| builder.mul_const(x.0, witness_map[&x.1]))
    //         .collect();

    //     let sum_a = builder.add_many(targets_a);

    //     let has_unknown_B = constraint.1.iter().any(|x| !witness_map.contains_key(&x.1));

    //     if has_unknown_B {
    //         panic!("B contains unknown");
    //     }

    //     let targets_b: Vec<Target> = constraint
    //         .1
    //         .iter()
    //         .map(|x| builder.mul_const(x.0, witness_map[&x.1]))
    //         .collect();

    //     let sum_b = builder.add_many(targets_b);

    //     let has_unknown_C = constraint.2.iter().any(|x| !witness_map.contains_key(&x.1));

    //     let a_x_b = builder.mul(sum_a, sum_b);

    //     if has_unknown_C {
    //         let position = constraint
    //             .2
    //             .iter()
    //             .position(|x| !witness_map.contains_key(&x.1))
    //             .unwrap();

    //         let neg_axb = builder.mul_const(F::NEG_ONE, a_x_b);

    //         let c: Vec<Target> = constraint
    //             .2
    //             .iter()
    //             .filter(|x| witness_map.contains_key(&x.1))
    //             .map(|x| builder.mul_const(x.0, witness_map[&x.1]))
    //             .collect();

    //         let sum_c = builder.add_many(c);
    //         let med = builder.add(neg_axb, sum_c);

    //         witness_map.insert(
    //             constraint.2[position].1,
    //             builder.mul_const(F::inverse(&constraint.2[position].0), med),
    //         );
    //     } else {
    //         let targets_c: Vec<Target> = constraint
    //             .2
    //             .iter()
    //             .map(|x| builder.mul_const(x.0, witness_map[&x.1]))
    //             .collect();

    //         let sum_c = builder.add_many(targets_c);

    //         let neg_c = builder.mul_const(F::NEG_ONE, sum_c);
    //         let result = builder.add(a_x_b, neg_c);
    //         builder.assert_zero(result);
    //     }
    // }

    // builder.register_public_input(witness_map[&1usize]);
    // builder.register_public_input(witness_map[&2usize]);
    // let mut pw = PartialWitness::new();
    // pw.set_target(witness_map[&0usize], F::from_canonical_u32(1));
    // pw.set_target(witness_map[&1usize], F::from_canonical_u32(531441));
    // pw.set_target(witness_map[&2usize], F::from_canonical_u32(3));

    // let data = builder.build::<C>();
    // let proof = data.prove(pw)?;
    // println!(
    //     "x^12 where x = {} is {}",
    //     proof.public_inputs[0].0 % 18446744069414584321u64,
    //     proof.public_inputs[1].0 % 18446744069414584321u64
    // );
    // data.verify(proof)
}
