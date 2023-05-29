# DendrETH: a smart contract implementation of the light client sync protocol

The low bandwidth requirements of the Ethereum [light client sync protocol](https://github.com/ethereum/annotated-spec/blob/master/altair/sync-protocol.md) create an unique opportunity for third-party blockchains to host smart contracts that implement a fully functional Ethereum light client.

Such a light client would be able to provide frequently updated information regarding recently finalized beacon chain block headers which in turn can be used to authenticate any portion of the Ethereum consensus or execution layer states through a chain of merkle proofs (or other commitment schemes in the future).

This would allow other smart contracts in the third-party blockchain to react to events happening in the Ethereum ecosystem, a functionality often described as a cross-blockchain bridge. Numerous bridge designs have already been proposed and implemented in the past, but a common shortcoming is that they rely on trusted oracles in their foundations, creating a systemic risk that the bridge may be compromised if the oracles are compromised.

A bridge based on the light client sync protocol will authenticate all data through the signatures of the Ethereum validators participating in the sync committees, significantly reducing the required level of trust in the bridge operator and limiting the potential for attacks.

The DendrETH project aims to develop highly efficient Ethereum light client implementations in the form of smart contracts for multiple third-party blockchains such as Solana, Polkadot, Cosmos, Cardano, Avalanche, Elrond, EOS, NEAR, Tezos and any EVM-compatible chain. It will explore and document different design trade-offs, based on the application of toolchains for fraud proofs and zero-knowledge proofs. The project is expected to produce a number of recommendations for changes in the Ethereum specifications that may improve the characteristic of the developed bridges in the future.

All developed software will be released under a FOSS license. The implemented bridges will be deployed and operated by at least one partnering large institutional operator ([Blockdaemon](https://blockdaemon.com/)).

## Team

The development is lead by [Zahary Karadjov](https://github.com/zah), currently serving as the [Nimbus](https://nimbus.team/) implementation team lead. The Nimbus team (and the rest of [Status](https://status.im/)) fully supports Zahary's participation in this project as it closely aligns with Status' mission to create highly efficient light clients.

The majority of the implementation work is carried out by an young team of blockchain developers
, selected with the notable help from [prof. Petko Ruskov](https://www.fmi.uni-sofia.bg/en/faculty/petko-ruskov-ruskov) at Sofia University and [Dr. Svetlin Nakov](https://cryptobook.nakov.com/) at [SoftUni](https://softuni.bg/):

- [Emil Ivanichkov](https://github.com/EmilIvanichkovv)
- [Dimo Dimov](https://github.com/Dimo99)
- [Simeon Armenchev](https://github.com/monyarm)
- [Yordan Miladinov](https://github.com/ydm)
- [Kristin Kirkov](https://github.com/kkirkov)

Former contributors include:

- [Georgi Georgiev](https://github.com/GeorgiGeorgiev7)
- [Georgi Chonkov](https://github.com/grc02)

The team is provided with mentorship consisting of frequent planning meetings, code reviews and direct implementation assistance from Zahary Karadjov, [Petar Kirov](https://github.com/PetarKirov) (former CTO of the [Jarvis Network](https://jarvis.network/), a company specializing in DeFi solutions for multiple chains) and [Rafael Belchior](https://github.com/rafaelapb), member of the Blockdaemon team, contributor and mentor at Hyperledger Cactus, and PhD student at [Técnico Lisboa](http://tecnico.ulisboa.pt/) and [INESC-ID](https://www.inesc-id.pt/), focusing on blockchain interoperability research.

### Suggested citation
Please use the following BibTex entry to cite this work while a paper is not available:

@report{dendreth2023, title = {DendrETH: Ethereum SNARK-Based Beacon Light Client for Multiple Blockchain Ecosystems}, url = {https://github.com/metacraft-labs/DendrETH}, number = {0}, institution = {Metacraft Labs and Blockdaemon}, author = {Ivanichkov, Emil and Dimov, Dimo and Armenchev, Simeon and Miladinov, Yordan and Kirkov, Kristin and Kirov, Petar and Karadjov, Zahary and Belchior, Rafael}, date = {2023}, }

## Timeline

### May 2022

Realised goals:

- Recruited the team and allowed all team members to build up all required pre-requisite knowledge for the project.

### June-September 2022

Goals:

- Develop a direct "naive" version of the Ethereum light client sync protocol for all targeted blockchains.
- Identify and document how all cryptographic operations can be mapped to available intrinsics/precompiles/etc. For each blockchain, catalogue the availability of toolchains for creating smart contracts based on fraud/fault proofs and zero-knowledge proofs that may be used in the next phase of the project.
- Develop automated components for maintaining a live bridge.
- Suggest new slashing conditions that will prevent the validators from producing extra-chain sync committee signatures that may be used by the bridge operator to carry out attacks.

Outcomes:

- We were able to deliver a highly efficient WebAssembly implementation of the light client syncing protocol, offering 36x size reduction when compared to the best prior art.
- We've surveyed the availability of the required cryptographic primitives in multiple blockchains and identified a promising solution based on existing zero-knowledge circuit for implementing parts of the required BLS signature verifications. We were able to further extend this circuit to cover other aspects of the light client syncing protocol and enclosed it with a relatively thin layer of Solidity code completing the implementation. For our Solidity work, we've leveraged prior research from Alex Stokes and the Darwinia team.
- We have published a complete archive of light client updates for Mainnet and Prater, as well as some simple tools for keeping the archive up-to-date in the future.
- We've presented an analysis regarding the safety and practicality of long-range light client syncing (including one-shot syncing through recursive zero-knowledge proofs) which is suggesting the addition of a very effective new slashing rule that can improve the security of light client bridges in the future.

Blockdaemon will deploy the developed contracts to testnets and estimate the long-term operational cost that will serve as baseline for measuring future improvements. The testnet bridges will be promoted in attempt to attract third-party developers of applications.

### October-December 2022

Goals:

- Complete the zero-knowledge circuit to cover the entire light client update verification logic, transitioning the state of the client directly from one header to another. This would allow the creation of a recursive circuit verifying multiple header-to-header transitions through a single proof. This capability will have useful applications for regular Ethereum clients, allowing them to instantly sync with the network after multiple months of being offline.

- Port the developed zero-knowledge circuit verification logic to WebAssembly to compare its efficiency against the developed direct implementation.

- Develop a zero-knowledge circuit verifying the Casper FFG finalization rules. Such a circuit will provide significantly higher crypto-economic security for trustless bridges, as it will be backed by the staked ETH of the entire validator set. Implementing the circuit will require use of more sophisticated recursive constructs for BLS aggregation and merkle root computations, but preliminary results suggest that a bridge will still be able to deliver updates with reasonable latency (e.g. 30 minutes).

- Develop a compatibility layer for bridge standards such as [TokenBridge](https://docs.tokenbridge.net/) and AMB to facilitate the creation of a trustless bridge between Ethereum and Gnosis Chain.

- Develop a succinct solution for the long-range attack problem, based on an one-shot syncing proof that keeps track of the observed minimal validator participation rate in the history. The solution will be based on the assumption that any long-range attack is likely to have a lower participation rate immediately after the forking point because the corrupted validators are likely to be active in both the real and the forged histories while the honest validators will be participating only in the real one. Thus, a client which is not subjected to an eclipse attack will be able to select the chain with higher minimal participation rate.

  In practice, clients employing one-shot syncing are likely to develop various heuristics for determining a safe syncing distance based on the demonstrated minimal participation rate. For example, if your client is 6 months behind the head block and you are presented with a proof that the minimal participation rate during this period was 97% (which has been historically true for mainnet), you can have high assurance that this is not a long-range attack because the validators are likely to be corrupted only if they have exited, but exiting 97% of the validators requires more than 6 months. In practice, it's also reasonable to assume that a certain percentage of the validators are non-corruptible which will make the 97% participation rate an even stronger guarantee.

- Port the developed contracts to all targeted blockchains.

- Attempt to verify the correctness of the developed circuits through formal methods and comprehensive tests.

Outcomes:

- A prototype for one-shot syncing based on a recursive zero-knowledge circuit was delivered in https://github.com/metacraft-labs/DendrETH/pull/58.

- Groth16 verifiers in [Rust](https://github.com/metacraft-labs/DendrETH/tree/main/beacon-light-client/circom/rust-verifier) and [Nim](https://github.com/metacraft-labs/DendrETH/pull/61) were added to the codebase and work is underway to build verifying smart contracts for Solana and Cosmos.

### 2023

Goals:

- Develop easy-to-use relay node software package capable of:

  - Monitoring the Ethereum network for light client updates.
  - Generating corresponding zero-knowledge proofs.
  - Executing transactions against the deployed smart contract to updates its state.

- Commission a security audit for the entire system.

- If necessary, create a custom Ethereum testnet, using a `hash_tree_root` based on hash function such as Pedersen Hash or Poseidon that is more friendly to zero-knowledge circuits.

Upon implementation of these goals, Blockdaemon commits to operate, promote, and maintain the developed bridges in all official networks. Furthermore, Blockdaemon commits to create and promote several innovative projects backed by academic research, on top of the provided trustless oracle:

1. Blockchain Migration

   The goal of the investigation is to build a proof of concept migrating data from a dApp on Ethereum 2.0 to a EVM-compatible chain in a trustless way (validating state migration via the oracle). This is project would be the first introducing trustless , automatic state migration across chains.

2. Integration with [Hyperledger Cactus](This is project would be the first introducing trustless , automatic state migration across chains).

   Hyperledger Cactus is the leading open-source, enterprise-grade interoperability project. Cactus aims to promote integration between enterprise systems and different blockchains. Cactus also provides support for developing infrastructure that integrates with Ethereum, and the developed oracle and bridge, including but not limited to: operators, relayers, and products on top of the bridge/oracle. The goal of this investigation is to diminish the entry barrier to enterprises wanting to use Ethereum 2.0. We can extend the current Ethereum connector to support Ethereum 2.0, and facilitate the integration with the developed bridges.

## Collaboration and Reporting

All development will take place in this Github repository and all communications related to the project will be carried out in a public [Telegram group](https://t.me/ProjectDendrETH). The team will publish monthly reports intended for the EF research team and other technical audiences, summarizing the most important findings and developments.

Upon important project milestones, Blockdaemon will publish a series of explainer articles promoting the project to the general public.
