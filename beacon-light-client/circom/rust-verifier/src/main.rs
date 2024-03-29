use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use serde::Deserialize;
use serde_json;
use std::env;
use std::fs::read_to_string;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct RawCircuitProof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct RawVerificationKey {
    vk_alpha_1: Vec<String>,
    vk_beta_2: Vec<Vec<String>>,
    vk_gamma_2: Vec<Vec<String>>,
    vk_delta_2: Vec<Vec<String>>,
    #[serde(rename = "IC")]
    ic: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct CircuitProof {
    a: G1Affine,
    b: G2Affine,
    c: G1Affine,
}

#[derive(Debug)]
pub struct CircuitVerifyingKey {
    alpha_g1: G1Affine,
    beta_g2: G2Affine,
    gamma_g2: G2Affine,
    delta_g2: G2Affine,
    gamma_abc_g1: Vec<G1Affine>,
}

fn fq_from_str(s: String) -> Fq {
    return Fq::from_str(&s).unwrap();
}

pub fn fr_from_str(s: String) -> Fr {
    return Fr::from_str(&s).unwrap();
}

pub fn g1_from_str(g1: &Vec<String>) -> G1Affine {
    let x = fq_from_str(g1[0].clone());
    let y = fq_from_str(g1[1].clone());
    let z = fq_from_str(g1[2].clone());
    return G1Affine::from(G1Projective::new(x, y, z));
}

pub fn g2_from_str(g2: &Vec<Vec<String>>) -> G2Affine {
    let c0 = fq_from_str(g2[0][0].clone());
    let c1 = fq_from_str(g2[0][1].clone());
    let x = Fq2::new(c0, c1);

    let c0 = fq_from_str(g2[1][0].clone());
    let c1 = fq_from_str(g2[1][1].clone());
    let y = Fq2::new(c0, c1);

    let c0 = fq_from_str(g2[2][0].clone());
    let c1 = fq_from_str(g2[2][1].clone());
    let z = Fq2::new(c0, c1);

    return G2Affine::from(G2Projective::new(x, y, z));
}

pub fn read_input_from_json(input_str: &str) -> Vec<Fr> {
    let params: Vec<String> = serde_json::from_str(&input_str).expect("Unable to parse");
    let mut ret = Vec::new();
    for param in params {
        ret.push(fr_from_str(param));
    }
    return ret;
}

impl CircuitVerifyingKey {
    pub fn read_input_from_json(input_str: &str) -> Self {
        let params: RawVerificationKey = serde_json::from_str(&input_str).expect("Unable to parse");

        let alpha_g1 = g1_from_str(&params.vk_alpha_1);
        let beta_g2 = g2_from_str(&params.vk_beta_2);
        let gamma_g2 = g2_from_str(&params.vk_gamma_2);
        let delta_g2 = g2_from_str(&params.vk_delta_2);
        let gamma_abc_g1: Vec<G1Affine> = params.ic.iter().map(|x| g1_from_str(&x)).collect();

        return CircuitVerifyingKey {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
        };
    }
}

impl From<CircuitVerifyingKey> for ark_groth16::VerifyingKey<Bn254> {
    fn from(src: CircuitVerifyingKey) -> ark_groth16::VerifyingKey<Bn254> {
        ark_groth16::VerifyingKey {
            alpha_g1: src.alpha_g1,
            beta_g2: src.beta_g2,
            gamma_g2: src.gamma_g2,
            delta_g2: src.delta_g2,
            gamma_abc_g1: src.gamma_abc_g1.into_iter().map(Into::into).collect(),
        }
    }
}

impl CircuitProof {
    pub fn read_input_from_json(input_str: &str) -> Self {
        let params: RawCircuitProof = serde_json::from_str(&input_str).expect("Unable to parse");

        // Parse pi_a
        let a: G1Affine = g1_from_str(&params.pi_a);

        // Parse pi_b
        let b: G2Affine = g2_from_str(&params.pi_b);

        // Parse pi_c
        let c: G1Affine = g1_from_str(&params.pi_c);

        return CircuitProof { a, b, c };
    }
}

impl From<CircuitProof> for ark_groth16::Proof<Bn254> {
    fn from(src: CircuitProof) -> ark_groth16::Proof<Bn254> {
        ark_groth16::Proof {
            a: src.a,
            b: src.b,
            c: src.c,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_str = read_to_string(&args[1]).unwrap();
    let res = CircuitVerifyingKey::read_input_from_json(&input_str);
    let out = ark_groth16::VerifyingKey::from(res);
    let pvk = ark_groth16::prepare_verifying_key(&out);

    let pub_input_str = read_to_string(&args[2]).unwrap();
    let pub_input = read_input_from_json(&pub_input_str);

    let proof_str = read_to_string(&args[3]).unwrap();
    let proof = ark_groth16::Proof::from(CircuitProof::read_input_from_json(&proof_str));

    if ark_groth16::verify_proof(&pvk, &proof, &pub_input).unwrap() {
        println!("OK!");
    } else {
        eprintln!("Invalid proof");
    }
}
