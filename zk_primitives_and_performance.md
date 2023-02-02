# Zero knowledge proofing systems measurements

## Primitives we need

|           | Circom groth16                             | Plonky2                                        | PIL                                 |
| --------- | ------------------------------------------ | ---------------------------------------------- | ----------------------------------- |
| SHA-256   | <https://github.com/yi-sun/circom-pairing> | <https://github.com/polymerdao/plonky2-sha256> |                                     |
| BLS12-381 | <https://github.com/yi-sun/circom-pairing> | Doesn't exist                                  | <https://github.com/puma314/blspil> |
| Pedersen  | circomlib                                  | Doesn't exist                                  |                                     |

- Anything that exists in circom should be achiavable in PIL as it should be able to describe the correctness of an entire PlonK circuit

## Primitives performance

### Circom

|                           | verify signature | elliptic curve add (2 points) | SHA-256 (1024bits) | pairing (groth16 recursive verification) | Pedersen (1024 bits) |
| ------------------------- | ---------------- | ----------------------------- | ------------------ | ---------------------------------------- | -------------------- |
| Constraints               | 19175103         | 11779                         | 89609              | 19661379                                 | 1816                 |
| Witness generation        | 1m               | 137ms                         | 110ms              | 1.1m                                     | 1ms                  |
| Proving time (rapidsnark) | 40s              | 14s                           | 14.5s              | 40s                                      | 100ms                |

### Plonky2

|              | verify signature | elliptic curve add | SHA-256 (1024bits) | plonky2 recursive verification | Pedersen       |
| ------------ | ---------------- | ------------------ | ------------------ | ------------------------------ | -------------- |
| Proving time | doesn't exist    | doesnt'exist       | 1.5342s            | 0.7393s                        | doesn't exists |
