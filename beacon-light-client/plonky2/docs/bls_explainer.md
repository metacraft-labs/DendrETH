For the bls signature verification we rely on https://github.com/Electron-Labs/starky_bls12_381

The bls signature verification happens in the following way you have as inputs
Pubkey, signature and message
The pubkey is just a compressed version of BLS G1 point, the signature is compressed version of bls G2 point and the message gets hashed to a G2 point using hash_to_curve

The signiture verification happens in the following way
pairing_precompute is ran over the message and the signature points

Then miller_loop is performed with the (pubkey, message) and (neg_generator_point, signature)
after that fp12mull is performed on both miller loop results and a final exponentiate is performed on the fp12mul the signiture is valid if the final_exponentiate results in Fp12::one().

For reference: https://github.com/Electron-Labs/starky_bls12_381

In starky_bls12_381 calc_pairing_precomp, miller_loop, fp12_mul, final_exponentiate are all separate starky circuits. To generate a proof of valid BLS signature what we do is we run them with the proper inputs wrap them in a plonky2 proofs and run one ours wrapping plonky2 circuits that should
return as public_inputs pubkey, signature and msg_targets and is_valid_signature.
To do that the circuit internaly accepts all the recursive proofs verifies them and connects that the result of each function is used as input to the other.
And that the points actually match the bytes passed to the circuit.
