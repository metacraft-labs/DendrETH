pragma circom 2.0.3;

include "../../circuits/proof_more_efficient.circom";

component main { public [ hash ] } = ProofMoreEfficient(512);
