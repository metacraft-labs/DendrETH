use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use num_bigint::BigUint;

pub struct ProofData {
    pub proof: ark_groth16::Proof<Bn254>,
    pub pub_input: [Fr; 4],
}

impl ProofData {
    pub fn unpack(input: &[u8; 384]) -> Box<ProofData> {
        let a1 = Fq::from(BigUint::from_bytes_le(&input[0..32]));
        let a2 = Fq::from(BigUint::from_bytes_le(&input[32..64]));

        let a = G1Affine::from(G1Projective::new(a1, a2, Fq::from(1)));

        let b11 = Fq::from(BigUint::from_bytes_le(&input[64..96]));
        let b12 = Fq::from(BigUint::from_bytes_le(&input[96..128]));

        let b21 = Fq::from(BigUint::from_bytes_le(&input[128..160]));
        let b22 = Fq::from(BigUint::from_bytes_le(&input[160..192]));

        let b = G2Affine::from(G2Projective::new(
            Fq2::new(b11, b12),
            Fq2::new(b21, b22),
            Fq2::new(Fq::from(1), Fq::from(0)),
        ));

        let c1 = Fq::from(BigUint::from_bytes_le(&input[192..224]));
        let c2 = Fq::from(BigUint::from_bytes_le(&input[224..256]));

        let c = G1Affine::from(G1Projective::new(c1, c2, Fq::from(1)));

        let pub_input1 = Fr::from(BigUint::from_bytes_le(&input[256..288]));
        let pub_input2 = Fr::from(BigUint::from_bytes_le(&input[288..320]));
        let pub_input3 = Fr::from(BigUint::from_bytes_le(&input[320..352]));
        let pub_input4 = Fr::from(BigUint::from_bytes_le(&input[352..384]));

        Box::new(ProofData {
            proof: ark_groth16::Proof { a, b, c },
            pub_input: [pub_input1, pub_input2, pub_input3, pub_input4]
        })
    }
}
