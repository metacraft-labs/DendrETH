import bncurve/groups
import std/json
import std/strformat

iterator `...`*[T](a: T, b: T): T =
  var res: T = T(a)
  while res <= b:
    yield res
    inc res

let vk = parseFile("../../beacon-light-client/circom/scripts/light_client_recursive/vkey.json");

let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][0][0].str),  c1: FQ.fromString(vk["vk_gamma_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][1][0].str), c1: FQ.fromString(vk["vk_gamma_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")));
let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_delta_2"][0][0].str),  c1: FQ.fromString(vk["vk_delta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_delta_2"][1][0].str), c1: FQ.fromString(vk["vk_delta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")));
let vkAlpha1 = Point[G1](x: FQ.fromString(vk["vk_alpha_1"][0].str), y: FQ.fromString(vk["vk_alpha_1"][1].str), z: FQ.fromString("1"));
let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_beta_2"][0][0].str),  c1: FQ.fromString(vk["vk_beta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_beta_2"][1][0].str), c1: FQ.fromString(vk["vk_beta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")));

let ic0 = Point[G1](x: FQ.fromString(vk["IC"][0][0].str), y: FQ.fromString(vk["IC"][0][1].str), z: FQ.fromString("1"));
let ic1 = Point[G1](x: FQ.fromString(vk["IC"][1][0].str), y: FQ.fromString(vk["IC"][1][1].str), z: FQ.fromString("1"));
let ic2 = Point[G1](x: FQ.fromString(vk["IC"][2][0].str), y: FQ.fromString(vk["IC"][2][1].str), z: FQ.fromString("1"));
let ic3 = Point[G1](x: FQ.fromString(vk["IC"][3][0].str), y: FQ.fromString(vk["IC"][3][1].str), z: FQ.fromString("1"));
let ic4 = Point[G1](x: FQ.fromString(vk["IC"][4][0].str), y: FQ.fromString(vk["IC"][4][1].str), z: FQ.fromString("1"));

for i in 291...533: 
    let proof = parseFile(fmt"../../vendor/eth2-light-client-updates/mainnet/proofs/proof{i}.json");
    let public = parseFile(fmt"../../vendor/eth2-light-client-updates/mainnet/proofs/public{i}.json");

    let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"));
    let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")));
    let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"));

    let pubInput1 = Fr.fromString(public[0].str);
    let pubInput2 = Fr.fromString(public[1].str);
    let pubInput3 = Fr.fromString(public[2].str);
    let pubInput4 = Fr.fromString(public[3].str);

    var preparedInputs = ic0;
    preparedInputs = preparedInputs + (ic1 * pubInput1);
    preparedInputs = preparedInputs + (ic2 * pubInput2);
    preparedInputs = preparedInputs + (ic3 * pubInput3);
    preparedInputs = preparedInputs + (ic4 * pubInput4);

    let aBPairing = pairing(a, b);
    let alphaBetaPairingP = pairing(vkAlpha1, vkBeta2);
    let preparedInputsGammaPairing = pairing(preparedInputs, vkGamma2);
    let proofCVkDeltaPairing = pairing(c, vkDelta2);

    let result = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

    if aBPairing == result:
        echo fmt"Ok, {i}";
    else:
        echo fmt"Wrong proof, {i}";
