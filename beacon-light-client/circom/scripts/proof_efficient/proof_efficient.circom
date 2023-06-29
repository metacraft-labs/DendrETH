pragma circom 2.1.5;

include "../../circuits/proof_efficient.circom";

component main { public [ bitmask, signature, hash ] } = ProofEfficient(512);
