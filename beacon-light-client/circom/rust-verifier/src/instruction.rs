use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use num_bigint::BigUint;

pub struct ProofData {
    pub proof: ark_groth16::Proof<Bn254>,
    pub pub_input: [Fr; 4],
    pub pvk: ark_groth16::PreparedVerifyingKey<Bn254>,
}

impl ProofData {
    pub fn unpack(input: &[u8; 384]) -> Self {
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

        let vk_data = [
            226, 242, 109, 190, 162, 153, 245, 34, 59, 100, 108, 177, 251, 51, 234, 219, 5, 157,
            148, 7, 85, 157, 116, 65, 223, 217, 2, 227, 167, 154, 77, 45, 38, 25, 77, 0, 255, 202,
            118, 240, 1, 3, 35, 25, 10, 131, 137, 206, 69, 227, 159, 32, 96, 236, 216, 97, 176,
            206, 55, 60, 80, 221, 190, 20, 171, 183, 61, 193, 127, 188, 19, 2, 30, 36, 113, 224,
            192, 139, 214, 125, 132, 1, 245, 43, 115, 214, 208, 116, 131, 121, 76, 173, 71, 120,
            24, 14, 12, 6, 243, 59, 188, 76, 121, 169, 202, 222, 242, 83, 166, 128, 132, 211, 130,
            241, 119, 136, 248, 133, 201, 175, 209, 118, 247, 203, 47, 3, 103, 9, 200, 206, 208,
            122, 84, 6, 127, 213, 169, 5, 234, 62, 198, 183, 150, 248, 146, 145, 47, 77, 210, 35,
            49, 49, 199, 168, 87, 164, 177, 193, 57, 23, 167, 70, 35, 17, 77, 154, 166, 157, 55,
            13, 122, 107, 196, 222, 253, 170, 60, 140, 63, 217, 71, 232, 245, 153, 74, 112, 138,
            224, 209, 251, 76, 48, 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212,
            34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194,
            18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49,
            183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230,
            76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235,
            109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179,
            112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137,
            6, 9, 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121,
            68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183,
            133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114,
            58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67,
            12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165,
            94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188,
            149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9, 237, 99, 220,
            94, 39, 135, 37, 221, 102, 142, 179, 55, 195, 241, 118, 79, 229, 113, 150, 12, 35, 69,
            0, 159, 204, 165, 252, 51, 7, 231, 72, 27, 141, 105, 89, 161, 2, 140, 214, 119, 86, 90,
            24, 147, 166, 217, 78, 238, 214, 233, 47, 6, 102, 213, 119, 67, 58, 215, 239, 124, 27,
            109, 50, 4, 241, 35, 39, 32, 78, 216, 141, 120, 138, 150, 95, 15, 38, 85, 80, 33, 23,
            138, 235, 114, 216, 145, 240, 236, 22, 6, 213, 110, 186, 220, 128, 45, 38, 53, 228, 78,
            180, 2, 144, 136, 24, 63, 73, 70, 89, 53, 249, 186, 53, 176, 204, 37, 244, 136, 162,
            134, 170, 191, 28, 40, 125, 191, 232, 21, 6, 200, 155, 171, 154, 14, 31, 130, 86, 171,
            202, 145, 106, 91, 22, 51, 94, 114, 214, 253, 116, 226, 128, 127, 130, 207, 169, 74,
            184, 181, 100, 21, 154, 228, 104, 27, 34, 174, 210, 113, 211, 188, 190, 156, 232, 127,
            230, 64, 72, 133, 38, 19, 92, 20, 37, 172, 88, 48, 120, 191, 252, 133, 66, 7, 63, 193,
            41, 208, 179, 108, 94, 148, 160, 37, 65, 123, 239, 32, 177, 239, 96, 98, 67, 15, 99,
            17, 193, 180, 90, 105, 110, 151, 144, 167, 31, 29, 146, 251, 87, 240, 185, 204, 84,
            174, 18, 195, 18, 252, 78, 8, 194, 225, 41, 44, 225, 146, 250, 116, 27, 34, 48, 207,
            111, 239, 43, 126, 184, 44, 106, 66, 201, 64, 190, 138, 94, 42, 213, 51, 171, 52, 241,
            140, 113, 168, 14, 232, 145, 112, 97, 111, 86, 191, 26, 226, 144, 217, 204, 143, 164,
            27, 96, 199, 76, 6, 216, 93, 245, 61, 114, 199, 169, 128, 55, 68, 163, 39, 49, 66, 5,
            223, 76, 175, 38, 38, 250, 152, 60, 55, 81, 168, 211, 22,
        ];

        let vk: ark_groth16::VerifyingKey<Bn254> = ark_groth16::VerifyingKey {
            alpha_g1: G1Affine::from(G1Projective::new(
                Fq::from(BigUint::from_bytes_le(&vk_data[0..32])),
                Fq::from(BigUint::from_bytes_le(&vk_data[32..64])),
                Fq::from(1),
            )),
            beta_g2: G2Affine::from(G2Projective::new(
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[64..96])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[96..128])),
                ),
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[128..160])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[160..192])),
                ),
                Fq2::new(Fq::from(1), Fq::from(0)),
            )),
            gamma_g2: G2Affine::from(G2Projective::new(
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[192..224])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[224..256])),
                ),
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[256..288])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[288..320])),
                ),
                Fq2::new(Fq::from(1), Fq::from(0)),
            )),
            delta_g2: G2Affine::from(G2Projective::new(
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[320..352])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[352..384])),
                ),
                Fq2::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[384..416])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[416..448])),
                ),
                Fq2::new(Fq::from(1), Fq::from(0)),
            )),
            gamma_abc_g1: vec![
                G1Affine::from(G1Projective::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[448..480])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[480..512])),
                    Fq::from(1),
                )),
                G1Affine::from(G1Projective::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[512..544])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[544..576])),
                    Fq::from(1),
                )),
                G1Affine::from(G1Projective::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[576..608])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[608..640])),
                    Fq::from(1),
                )),
                G1Affine::from(G1Projective::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[640..672])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[672..704])),
                    Fq::from(1),
                )),
                G1Affine::from(G1Projective::new(
                    Fq::from(BigUint::from_bytes_le(&vk_data[704..736])),
                    Fq::from(BigUint::from_bytes_le(&vk_data[736..768])),
                    Fq::from(1),
                )),
            ],
        };

        let pvk = ark_groth16::prepare_verifying_key(&vk);

        ProofData {
            proof: ark_groth16::Proof { a, b, c },
            pub_input: [pub_input1, pub_input2, pub_input3, pub_input4],
            pvk: pvk,
        }
    }
}
