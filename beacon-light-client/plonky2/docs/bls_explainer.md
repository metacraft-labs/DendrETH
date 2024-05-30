The BLS signature verification is based on third-party circuits located at https://github.com/Electron-Labs/starky_bls12_381 and works in the following way:

We have `pubkey`, `signature` and `message` as inputs.
The `pubkey` is just a compressed version of BLS G1 point, the `signature` is a compressed version of BLS G2 point and the `message` gets hashed to a G2 point using `hash_to_curve`.

The signature verification happens in the following way:

1) `pairing_precompute` is ran over the message and the signature points
2) Then `miller_loop` is performed with the `(pubkey, message)` and `(neg_generator_point, signature)`
3) After that `fp12mull` is performed on both miller loop results and `final_exponentiate` is executed on the `fp12mul` result. The signature is valid if the `final_exponentiate` results in `Fp12::one()`.

In starky_bls12_381, `calc_pairing_precomp`, `miller_loop`, `fp12_mul`, `final_exponentiate` are all separate starky circuits.

To generate a proof of valid BLS signature, we run them with the proper inputs and wrap them in a plonky2 proofs which are then passed to an aggregating plonky2 circuit that has as public inputs/outputs `pubkey`, `signature` and `msg_targets` and `is_valid_signature`. To achieve its goal, the circuit internally accepts all the sub-proofs, verifies them and checks that the result of each function is used as input to the other and that the points actually match the bytes passed to the circuit.
