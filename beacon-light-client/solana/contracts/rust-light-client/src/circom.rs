use std::str::FromStr;

use crate::instruction::ProofData;
use ark_bn254::{Bn254, Fq, Fq2, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_groth16::VerifyingKey;
use solana_program::msg;

pub fn run_verifier(proof_data: &ProofData) -> bool {
    // let vk: VerifyingKey<Bn254> = VerifyingKey {
    //     alpha_g1: G1Affine::from(G1Projective::new(
    //         Fq::from_str(
    //             "20491192805390485299153009773594534940189261866228447918068658471970481763042",
    //         )
    //         .unwrap(),
    //         Fq::from_str(
    //             "9383485363053290200918347156157836566562967994039712273449902621266178545958",
    //         )
    //         .unwrap(),
    //         Fq::from(1),
    //     )),
    //     beta_g2: G2Affine::from(G2Projective::new(
    //         Fq2::new(
    //             Fq::from_str(
    //                 "6375614351688725206403948262868962793625744043794305715222011528459656738731",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "4252822878758300859123897981450591353533073413197771768651442665752259397132",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(
    //             Fq::from_str(
    //                 "10505242626370262277552901082094356697409835680220590971873171140371331206856",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "21847035105528745403288232691147584728191162732299865338377159692350059136679",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(Fq::from(1), Fq::from(0)),
    //     )),
    //     gamma_g2: G2Affine::from(G2Projective::new(
    //         Fq2::new(
    //             Fq::from_str(
    //                 "10857046999023057135944570762232829481370756359578518086990519993285655852781",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "11559732032986387107991004021392285783925812861821192530917403151452391805634",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(
    //             Fq::from_str(
    //                 "8495653923123431417604973247489272438418190587263600148770280649306958101930",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "4082367875863433681332203403145435568316851327593401208105741076214120093531",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(Fq::from(1), Fq::from(0)),
    //     )),
    //     delta_g2: G2Affine::from(G2Projective::new(
    //         Fq2::new(
    //             Fq::from_str(
    //                 "10857046999023057135944570762232829481370756359578518086990519993285655852781",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "11559732032986387107991004021392285783925812861821192530917403151452391805634",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(
    //             Fq::from_str(
    //                 "8495653923123431417604973247489272438418190587263600148770280649306958101930",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "4082367875863433681332203403145435568316851327593401208105741076214120093531",
    //             )
    //             .unwrap(),
    //         ),
    //         Fq2::new(Fq::from(1), Fq::from(0)),
    //     )),
    //     gamma_abc_g1: vec![
    //         G1Affine::from(G1Projective::new(
    //             Fq::from_str(
    //                 "12341254398012831539511529514141920531233310925559640868133205893740926624749",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "1898346778999733876099328904356803438781336221250567894820659652669409421709",
    //             )
    //             .unwrap(),
    //             Fq::from(1),
    //         )),
    //         G1Affine::from(G1Projective::new(
    //             Fq::from_str(
    //                 "20581758020956979671791380396676180914933879489933044097984142919872524264433",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "9909799947047067906035029346433967103598628542141844188201546463877414532390",
    //             )
    //             .unwrap(),
    //             Fq::from(1),
    //         )),
    //         G1Affine::from(G1Projective::new(
    //             Fq::from_str(
    //                 "9676508711308354042906200636781077532751603523420398102911817991603787384838",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "3283726592693011886356839062982995108864203785773230930373125221081605006490",
    //             )
    //             .unwrap(),
    //             Fq::from(1),
    //         )),
    //         G1Affine::from(G1Projective::new(
    //             Fq::from_str(
    //                 "13173001357742665986032466031029094851321843920842980071903261453747149717823",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "20227736002115979502147501163867970082744770988953245358764176832339037911954",
    //             )
    //             .unwrap(),
    //             Fq::from(1),
    //         )),
    //         G1Affine::from(G1Projective::new(
    //             Fq::from_str(
    //                 "12503202302840927457518240189095194534947514058111893826856048640111724282474",
    //             )
    //             .unwrap(),
    //             Fq::from_str(
    //                 "10324849082459144926597406829440562859648647791510582416705714907864823023456",
    //             )
    //             .unwrap(),
    //             Fq::from(1),
    //         )),
    //     ],
    // };
    let alpha_g1 =  G1Affine::from(G1Projective::new(
        Fq::from_str(
            "20491192805390485299153009773594534940189261866228447918068658471970481763042",
        )
        .unwrap(),
        Fq::from_str(
            "9383485363053290200918347156157836566562967994039712273449902621266178545958",
        )
        .unwrap(),
        Fq::from(1),
    ));

    msg!("do tuka go smetnah {}", alpha_g1);

    // let pvk = ark_groth16::prepare_verifying_key(&vk);

    // if ark_groth16::verify_proof(&pvk, &proof_data.proof, &proof_data.pub_input).unwrap() {
    //     return true;
    // } else {
    //     return false;
    // }

    return true;
}
