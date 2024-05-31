For the bls signature verification we rely on [starky implementation of bls12-381](https://github.com/Electron-Labs/starky_bls12_381)

# Structure

    plonky2
        |
        |-circuits
        |   |-bls12_381_circuit.rs
        |-circuits_executables
            |-bin
                |-calc_pairing_precomp_circuit_data_generation.rs
                |-calc_pairing_precomp.rs
                |-miller_loop_circuit_data_generation.rs
                |-miller_loop.rs
                |-final_exponentiate_circuit_data_generation.rs
                |-final_exponentiate.rs
                |-fp12_mul_circuit_data_generation.rs
                |-fp12_mul.rs
                |-bls_381_circuit_data_generation.rs
                |-bls_381_components_proofs.rs
                |-bls_381.rs



# Circuits
- `calc_pairing_precomp`

Calculates the pairing precomputation for both the `signature` and `message`. It takes the coefficients of their compressed versions as input and returns a vector of three `Fp2` coefficients for both.
- `miller_loop`

Performs the Miller loop algorithm. It takes the coefficients of the compressed points of the public key, the G1 generator point and the result of the pairing precomputation of `message`  and `signature` as input and returns the corresponding `Fp12` elements. There are two instances of the Miller loop: one for `(pubkey, message)` and another for `(neg_generator_point, signature)`.

- `fp12_mul`

Takes two `Fp12` elements as input and returns their product as another `Fp12` element.

- `final_exponentiate`

Takes the resulting `Fp12` element from `fp12_mul` as input, and returns the final exponentiation algorithm of it. The signature is considered valid if this result equals `Fp12::one()`.

To generate a proof of valid BLS signature, we run the Starky circuits with appropriate inputs. The result of each Starky circuit, which is of our interest, is its `StarkProofWithPublicInputs`, specifically the `public_inputs`. The public inputs of each Starky circuit are then passed through recursive proofs, whose public inputs are connected such that the output of each function is used as input in the other. Ensure that the points correspond to the bytes passed as input to the circuit.

# Workflow
The BLS signature verification process involves the following steps:

1. As parameters, we pass `pubkey`, `signature`, and `message`, where `pubkey` is a compressed version of an G1 elliptic curve point, the `signature` is a compressed version of an G2 elliptic curve point, and the `message` is hashed to obtain a G2 EC point using `hash_to_curve`.

2. The coefficients of the compressed versions of the `signature` and `message` are processed by `calc_pairing_precomp` circuit, resulting in a vector of three `Fp2` coefficients for both.

3. The resulting coefficients of both the `signature` and `message` are fed into the `miller_loop` circuit, alongside the coefficients of the compressed points of the public key and the G1 generator point. There are two instances of `miller_loop`: one for `(pubkey, message)` and another for `(neg_generator_point, signature)`.

4. The outputs of the two miller loops are two `Fp12` elements. These elements are then input into `fp12_mul` circuit, where they are multiplied together.

5. The result of `fp12_mul` is another `Fp12` element, which is then subjected to `final_exponentiation` circuit.

6. Lastly, the `signature` is considered valid if the final exponentiation's result equals `Fp12::one()`.

