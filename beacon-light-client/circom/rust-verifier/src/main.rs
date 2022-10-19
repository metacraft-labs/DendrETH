use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_groth16::Proof;
use serde::Deserialize;
use serde_json;
use std::env;
use std::fs::read_to_string;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct RawCircuitProof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
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

pub mod instruction;
use crate::instruction::ProofData;
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
fn main() {
    let instruction_data: [u8; 384] = [
        11, 7, 201, 86, 108, 234, 171, 101, 246, 129, 197, 161, 102, 172, 91, 179, 81, 130, 206,
        28, 90, 71, 4, 180, 18, 216, 27, 59, 84, 218, 167, 21, 4, 211, 41, 64, 124, 230, 84, 98,
        177, 136, 199, 242, 6, 187, 249, 62, 223, 232, 38, 220, 188, 79, 101, 32, 254, 212, 163,
        232, 22, 104, 54, 45, 66, 225, 164, 171, 51, 236, 65, 56, 254, 63, 89, 170, 204, 246, 95,
        40, 191, 112, 151, 112, 45, 129, 42, 118, 176, 213, 68, 209, 155, 225, 50, 43, 7, 64, 158,
        209, 169, 24, 119, 201, 66, 46, 156, 247, 232, 128, 187, 234, 134, 238, 203, 217, 175, 193,
        30, 222, 236, 132, 249, 240, 141, 196, 11, 48, 129, 41, 216, 181, 249, 78, 83, 145, 167,
        236, 102, 154, 233, 27, 249, 209, 231, 21, 61, 7, 27, 78, 251, 109, 78, 115, 164, 195, 245,
        64, 144, 34, 132, 45, 126, 240, 198, 46, 85, 216, 220, 89, 171, 145, 119, 169, 65, 103,
        216, 254, 230, 71, 249, 140, 165, 17, 174, 148, 143, 198, 210, 229, 71, 4, 156, 26, 229,
        99, 240, 243, 47, 180, 228, 233, 201, 38, 16, 116, 164, 1, 172, 46, 31, 253, 185, 3, 180,
        181, 46, 220, 114, 87, 157, 193, 34, 40, 193, 61, 223, 105, 41, 143, 121, 90, 40, 145, 149,
        150, 72, 180, 39, 17, 39, 187, 98, 168, 201, 171, 114, 10, 223, 184, 183, 213, 50, 65, 82,
        23, 150, 112, 31, 83, 62, 171, 83, 152, 247, 9, 154, 23, 30, 20, 123, 210, 101, 116, 20,
        25, 219, 193, 117, 212, 167, 68, 32, 174, 29, 167, 48, 13, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 227, 205, 135, 104, 194, 99,
        208, 121, 227, 109, 5, 212, 185, 131, 111, 101, 64, 109, 80, 254, 225, 178, 177, 145, 68,
        213, 197, 249, 169, 255, 108, 28, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let proof_data = ProofData::unpack(&instruction_data);

    if ark_groth16::verify_proof(&proof_data.pvk, &proof_data.proof, &proof_data.pub_input).unwrap() {
        println!("OK!");
    } else {
        eprintln!("Invalid proof");
    }
}
