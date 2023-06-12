pragma circom 2.1.5;

include "../../circuits/proof_more_efficient.circom";

component main { public [ hash ] } = ProofMoreEfficient(512);
