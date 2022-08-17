# Survey existing EVM light client implementations for other Blockchains

## Solana - trustless bridge by nilFoundation

### Main characteristics

- Presented as *the first completely trustless bridge* since it does not require any relays, tokens or validators

- Gives the opportunity to build various applications ot top of it (such as tokens exchange, etc.)

- Uses a general-purpose zero-knowledge proof scheme called [PLONK](https://www.youtube.com/watch?v=RUZcam_jrz0&list=PLBJMt6zV1c7Gh9Utg-Vng2V6EYVidTFCC)

- Using a ton of complex cryptographic functions: 
   - __Poseidon Circuit__ (for state Merkle-tree and proof generation)
   - __SHA2-256 Circuit__ and __SHA2-512 Circuit__ (for block headers and transactions hashes verification)
   - __Merkle Tree Circuit__
   - __Ed25519 Circuit__ (for validators signature verification)
   - __Correct Validator Set Proof Circuit__

- [Truffle project repo](https://github.com/NilFoundation/evm-placeholder-verification) contains some small issues in the block gas limit setting in truffle-config.js, contracts compilation and tests running. To fix the first 2:
   - reduce the block gas limit in the `development` network to 30M on [line 48 in truffle-config.js](https://github.com/NilFoundation/evm-placeholder-verification/blob/master/truffle-config.js#L48)
   - Use solc-select or enable docker if you have it installed when compiling and testing contracts - [reference](https://github.com/trufflesuite/truffle/issues/3076#issuecomment-634937658)

- There are a total of 39 tests of which 32 passing and 7 failing:
```
  Contract: Permutation argument
    1) Case 1
    2) Case 2
  Contract: Placeholder verifier unified addition component
    3) Case 1
    4) Case 2
  Contract: Unified addition gate argument
    5) Case 1
    6) Case 2
    7) Case 3
```

### Resources

- [Solana docs](https://docs.solana.com/proposals/simple-payment-and-state-verification#light-clients)

- [Nill Foundation blog](https://blog.nil.foundation/)

- [Understanding PLONK](https://vitalik.ca/general/2019/09/22/plonk.html) blog post by Vitalik

- [PLONK explanational videos](https://www.youtube.com/watch?v=RUZcam_jrz0&list=PLBJMt6zV1c7Gh9Utg-Vng2V6EYVidTFCC)

## Polkadot

### Main characteristics

- Easy setup of mixed Hardhat, Truffle, ethers.js and web3.js [project repository](https://github.com/Snowfork/snowbridge/tree/main/ethereum)

- Deployment gas costs can be reduced with approximately [4-4.25%](https://github.com/Snowfork/snowbridge/compare/main...GeorgiGeorgiev7:snowbridge:gas-optimization) by appling some basic [gas optimization patterns](https://github.com/metacraft-labs/DendrETH/tree/main/survey/evm-light-clients/gas-optimization-reports/snowbridge.md)

- Not many unit tests, only basic and sample ones. A good idea will be to migrate the whole project to a Hardhat-only or Foundry based environent so more efficient tests can be written.

- There are several [single points of failure](https://github.com/Snowfork/snowbridge/blob/main/ethereum/contracts/ETHApp.sol#L154) that if exploited may lead to breaking the whole bridge. After testing the `handleReward` function using Foundry's fuzzing tests with 10_000_000 runs (different random input scenarios) the result is:
```
Running 1 test for test\ETHApp.t.sol:ETHAppTest
[PASS] handleReward(address,uint128) (runs: 10000000, μ: 38945, ~: 40228)
Test result: ok. 1 passed; 0 failed; finished in 1070.52s
```

- Contracts seem to work fine, but still there is a lot of room for improvements

### Resources

- [Snowbridge ethereum light client contracts](https://github.com/Snowfork/snowbridge/tree/main/ethereum)

- [Slightly (~4%) optimized fork](https://github.com/Snowfork/snowbridge/compare/main...GeorgiGeorgiev7:snowbridge:gas-optimization)

## Cosmos - Tendermint light client sync

### Main characteristics

- Two branches: `main` and `optimized` - the optimized version of the protocol is made to fit the [Celo blockchain](https://celo.org/) block gas limit of 20M while keeping all functionalities

- Fitting in the range below 20M block gas limit means that it can also fit in Ethereum (30M), BSC (~85M), Algorand (100M) and Phantom (31M), but not in Avalanche (8M)

- Supports both the Secp256k1 and Ed25519 curves

- Two methods to sync the light client from the trusted header to the latest state: `sequential` and `skipping verifications`. \
 "Despite its simplicity, verifying headers sequentially is slow (due to signature verification) and requires downloading all intermediate headers."

- Weak subjectivity - first light client header is no older than one unbonding window and is being fetched from a trusted source (as is the case in [DendrETH](https://github.com/metacraft-labs/DendrETH/blob/main/docs/BEACON-REST-API.md))

- [README.md](https://github.com/ChorusOne/tendermint-sol#results-overview) includes information about the performance and gas usage of the mentioned branches (`main`/vanilla and `optimized`) and modes (`adjacent`  and `non-adjacent`) covering each of their weaknesses and strengths

- Some [gas optimizations](https://github.com/metacraft-labs/DendrETH/tree/main/survey/evm-light-clients/gas-optimization-reports/tendermint-sol.md) may be applied - mainly for-loop gas optimizations and using custom errors

### Resources

- An [article](https://medium.com/tendermint/everything-you-need-to-know-about-the-tendermint-light-client-f80d03856f98) by a former Tendermint dev about how the Tendermint Light Client works

- A [demo video](https://asciinema.org/a/456622) showing local deployment and quick setup

- [Required local setup](https://github.com/ChorusOne/tendermint-sol#setup)
