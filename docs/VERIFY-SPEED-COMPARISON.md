# Speed comparison of different libraries used for operations on the BN256 curve

## Time for pairing:

Time to do 5 pairings(of 2 points) or in the case of 'constantine',
'barretenberg' and ffjavascript - a multi pairing(of 10 points)

### Speed in release(milliseconds):

- bncurve: 24,8(6,4 for 1)
- bn-rust: 16.4(4.2 for 1)
- constantine: 0,8
- barretenberg 1.15

### Speed not in release(milliseconds):

- ffjavascript: 5-6
- bncurve: 238(62 for 1)
- bn-rust: 539.7(134.3 for 1)
- constantine: 5,1

## Full time to verify:

Full time needed for verification using our implementation of the verifier

### Speed in release(milliseconds):

- bncurve: 26
- bn-rust: 22
- constantine: 1,1

### Speed not in release(milliseconds):

- ffjavascript: 344
- bncurve: 245
- bn-rust: 745
- constantine: 7,5

## NIM

https://github.com/status-im/nim-bncurve - bncurve(forked)

https://github.com/mratsim/constantine - constantine(forked)
(https://github.com/metacraft-labs/constantine/tree/af2085ccfae77fd9f1f7755c62211ca229edeef3)

## RUST

https://github.com/zcash-hackworks/bn - bn-rust

## Javascript

https://github.com/iden3/ffjavascript - ffjavascript

## C++

https://github.com/AztecProtocol/barretenberg - barretenberg
