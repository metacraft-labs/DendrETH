pragma circom 2.0.3;

include "../../circuits/proof_efficient.circom";

component main { public [ bitmask, signature, hash ] } = ProofEfficient(2);
