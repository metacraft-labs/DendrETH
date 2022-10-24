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
        56, 132, 199, 69, 208, 238, 1, 248, 17, 3, 34, 59, 135, 163, 225, 23, 34, 79, 127, 153,
        238, 47, 184, 36, 153, 156, 163, 224, 131, 173, 179, 46, 246, 225, 217, 90, 9, 147, 97,
        128, 186, 15, 161, 188, 164, 217, 194, 195, 27, 159, 31, 161, 194, 61, 70, 181, 145, 250,
        212, 85, 188, 236, 163, 4, 208, 247, 187, 46, 172, 171, 91, 156, 139, 75, 215, 43, 168,
        121, 86, 36, 83, 38, 246, 114, 158, 105, 161, 14, 246, 80, 170, 62, 210, 42, 39, 16, 244,
        7, 187, 17, 106, 44, 94, 125, 117, 112, 51, 57, 115, 78, 122, 251, 235, 11, 72, 196, 176,
        10, 136, 70, 156, 127, 29, 101, 231, 221, 205, 16, 71, 41, 255, 23, 222, 126, 123, 143, 48,
        236, 160, 120, 31, 52, 165, 245, 251, 51, 110, 87, 170, 18, 187, 139, 189, 43, 95, 26, 251,
        106, 6, 17, 215, 102, 159, 217, 68, 79, 220, 4, 18, 146, 102, 230, 179, 154, 15, 174, 138,
        105, 195, 102, 66, 221, 75, 55, 83, 83, 142, 134, 117, 236, 61, 5, 65, 146, 211, 145, 122,
        243, 6, 57, 185, 222, 201, 111, 36, 220, 142, 1, 86, 55, 145, 8, 68, 35, 27, 117, 82, 146,
        14, 146, 147, 147, 62, 40, 99, 165, 152, 25, 143, 128, 100, 103, 159, 194, 77, 204, 24, 35,
        202, 125, 150, 184, 168, 105, 33, 186, 142, 199, 30, 197, 54, 96, 225, 244, 245, 47, 150,
        112, 31, 83, 62, 171, 83, 152, 247, 9, 154, 23, 30, 20, 123, 210, 101, 116, 20, 25, 219,
        193, 117, 212, 167, 68, 32, 174, 29, 167, 48, 13, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 227, 205, 135, 104, 194, 99, 208,
        121, 227, 109, 5, 212, 185, 131, 111, 101, 64, 109, 80, 254, 225, 178, 177, 145, 68, 213,
        197, 249, 169, 255, 108, 28, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let proof_data = ProofData::unpack(&instruction_data);

    if ark_groth16::verify_proof(&proof_data.pvk, &proof_data.proof, &proof_data.pub_input).unwrap()
    {
        println!("OK!");
    } else {
        eprintln!("Invalid proof");
    }
}
