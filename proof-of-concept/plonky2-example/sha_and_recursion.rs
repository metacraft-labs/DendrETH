use anyhow::Result;
use log::{Level, LevelFilter};
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, VerifierCircuitTarget};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;
use plonky2_sha256::circuit::{array_to_bits, make_circuits};
use sha2::{Digest, Sha256};

pub fn prove_sha256(msg: &[u8]) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    let hash = hasher.finalize();

    let msg_bits = array_to_bits(msg);
    let len = msg.len() * 8;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
    let targets = make_circuits(&mut builder, len as u64);
    let mut pw = PartialWitness::new();

    for i in 0..len {
        pw.set_bool_target(targets.message[i], msg_bits[i]);
    }

    let expected_res = array_to_bits(hash.as_slice());
    for i in 0..expected_res.len() {
        if expected_res[i] {
            // builder.assert_one(targets.digest[i].target);
            builder.register_public_input(targets.digest[i].target);
        } else {
            // builder.assert_zero(targets.digest[i].target);
            builder.register_public_input(targets.digest[i].target);
        }
    }

    println!(
        "Constructing inner proof with {} gates",
        builder.num_gates()
    );
    let data = builder.build::<C>();
    let timing = TimingTree::new("prove", Level::Debug);
    let proof = data.prove(pw).unwrap();
    timing.print();

    let timing = TimingTree::new("verify", Level::Debug);
    let res = data.verify(proof.clone());
    timing.print();

    let timing = TimingTree::new("recursive", Level::Debug);
    let mut recursive_builder =
        CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());
    let mut recursive_pw = PartialWitness::new();
    let inner_proof = proof.clone();
    let inner_vd = data.verifier_only;
    let inner_cd = data.common;

    let pt = recursive_builder.add_virtual_proof_with_pis::<C>(&inner_cd);

    recursive_pw.set_proof_with_pis_target(&pt, &inner_proof);
    let inner_data = VerifierCircuitTarget {
        constants_sigmas_cap: recursive_builder
            .add_virtual_cap(inner_cd.config.fri_config.cap_height),
        circuit_digest: recursive_builder.constant_hash(inner_vd.circuit_digest),
    };

    recursive_pw.set_cap_target(
        &inner_data.constants_sigmas_cap,
        &inner_vd.constants_sigmas_cap,
    );
    recursive_builder.verify_proof::<C>(&pt, &inner_data, &inner_cd);

    let recursive_target = make_circuits(&mut recursive_builder, 256);
    for i in 0..256 {
        // println!("{}", inner_proof.public_inputs[i].eq(&F::ZERO));
        recursive_pw.set_bool_target(
            recursive_target.message[i],
            !inner_proof.public_inputs[i].eq(&F::ZERO),
        );
    }

    for i in 0..256 {
        recursive_builder.register_public_input(recursive_target.digest[i].target);
    }
    let recursive_data = recursive_builder.build::<C>();
    timing.print();

    let timing = TimingTree::new("proverecursion", Level::Debug);
    let proof_recursion = recursive_data.prove(recursive_pw).unwrap();
    timing.print();

    let timing = TimingTree::new("verifyrecursion", Level::Debug);
    let res_rec = recursive_data.verify(proof_recursion.clone());
    timing.print();

    for i in 0..256 {
        print!("{}", proof_recursion.public_inputs[i]);
    }

    for i in 0..256 {
        print!("{}", proof.public_inputs[i]);
    }

    res_rec
}

fn main() -> Result<()> {
    // Initialize logging
    let mut builder = env_logger::Builder::from_default_env();
    builder.format_timestamp(None);
    builder.filter_level(LevelFilter::Debug);
    builder.try_init()?;

    const MSG_SIZE: usize = 128;
    let mut msg = vec![0; MSG_SIZE as usize];
    for i in 0..MSG_SIZE {
        msg[i] = i as u8;
    }
    prove_sha256(&msg)
}
