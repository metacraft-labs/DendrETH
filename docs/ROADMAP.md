# DendrETH: a smart contract implementation of the light client sync protocol

The low bandwidth requirements of the Ethereum [light client sync protocol](https://github.com/ethereum/annotated-spec/blob/master/altair/sync-protocol.md) create an unique opportunity for third-party blockchains to host smart contracts that implement a fully functional Ethereum light client.

Such a light client would be able to provide frequently updated information regarding recently finalized beacon chain block headers which in turn can be used to authenticate any portion of the Ethereum consensus or execution layer states through a chain of merkle proofs (or other commitment schemes in the future).

This would allow other smart contracts in the third-party blockchain to react to events happening in the Ethereum ecosystem, a functionality often described as a cross-blockchain bridge. Numerous bridge designs have already been proposed and implemented in the past, but a common shortcoming is that they rely on trusted oracles in their foundations, creating a systemic risk that the bridge may be compromised if the oracles are compromised.

A bridge based on the light client sync protocol will authenticate all data through the signatures of the Ethereum validators participating in the sync committees, significantly reducing the required level of trust in the bridge operator and limiting the potential for attacks.

The DendrETH project aims to develop highly efficient Ethereum light client implementations in the form of smart contracts for multiple third-party blockchains such as Solana, Polkadot, Cosmos, Cardano, Avalanche, Tezos and any EVM-compatible chain. It will explore and document different design trade-offs, based on the application of toolchains for fraud proofs and zero-knowledge proofs. The project is expected to produce a number of recommendations for changes in the Ethereum specifications that may improve the characteristic of the developed bridges in the future.

All developed software will be released under permissive license. The implemented bridges will be deployed and operated by at least one partnering large institutional operator ([Blockdaemon](https://blockdaemon.com/)).

## Team

The development will be lead by [Zahary Karadjov](https://github.com/zah), currently serving as the [Nimbus](https://nimbus.team/) implementation team lead. The Nimbus team (and the rest of [Status](https://status.im/)) fully supports Zahary's participation in this project as it closely aligns with Status' mission to create highly efficient light clients.

The majority of the implementation work will be carried out by an young team of blockchain developers
, selected with the notable help from [prof. Petko Ruskov](https://www.fmi.uni-sofia.bg/en/faculty/petko-ruskov-ruskov) at Sofia University and [Dr. Svetlin Nakov](https://cryptobook.nakov.com/) at [SoftUni](https://softuni.bg/):

* [Emil Ivanichkov](https://github.com/EmilIvanichkovv)
* [Simeon Armenchev](https://github.com/monyarm)
* [Dimo Dimov](https://github.com/Dimo99)
* [Georgi Chonkov](https://github.com/grc02)
* [Georgi Georgiev](https://github.com/GeorgiGeorgiev7)

The team will be provided with mentorship consisting of frequent planning meetings, code reviews and direct implementation assistance from Zahary Karadjov, [Petar Kirov](https://github.com/PetarKirov) (former CTO of the [Jarvis Network](https://jarvis.network/), a company specializing in DeFi solutions for multiple chains) and [Rafael Belchior](https://github.com/rafaelapb), member of the Blockdaemon team, contributor and mentor at Hyperledger Cactus, and PhD student at [Técnico Lisboa](http://tecnico.ulisboa.pt/) and [INESC-ID](https://www.inesc-id.pt/), focusing on blockchain interoperability research.

## Timeline

### May 2022

Goals:

* Recruit the team and conduct a series of introductory seminars for all required pre-requisite knowledge for the project.

### June-August 2022

Goals:

* Develop a direct "naive" version of the Ethereum light client sync protocol for all targeted blockchains.
* Identify and document how all cryptographic operations can be mapped to available intrinsics/precompiles/etc. For each blockchain, catalogue the availability of toolchains for creating smart contracts based on fraud/fault proofs and zero-knowledge proofs that may be used in the next phase of the project.
* Develop automated components for maintaining a live bridge.
* Suggest new slashing conditions that will prevent the validators from producing extra-chain sync committee signatures that may be used by the bridge operator to carry out attacks.


The naive implementation is expected to be costly to operate because the bridge operator will have to upload full light client updates in the form of blockchain transactions, but it will provide accurate data and a stable API for third-party applications that want to build upon the bridge.

Blockdaemon will deploy the developed contracts to testnets and estimate the long-term operational cost that will serve as baseline for measuring future improvements. The testnet bridges will be promoted in attempt to attract third-party developers of applications.

### September-December 2022

Goals:

* Improve the BLS support in third-party blockchains and fraud proof run-times by following the usual contribution process (for all networks where this was deemed necessary).

* Explore fraud proofs as a mechanism for compressing the required data that must be posted on chain.

  We believe that the base layer transactions need to include only the following data items:

  - beacon chain block header hash.
  - hash of the `LightClientUpdate` corresponding to specified block header.
  - slot number (not strictly necessary, but deemed useful enough).
  - finalized block header hash.

  A verifier will be able to use the following algorithm:

  - Attempt to recreate the `LightClientUpdate` from the indicated block header (a fully synced beacon node has all the required information).
  - If the locally computed update doesn't match the provided commitment hash, start the challenge process. The operator should not be able to produce an alternative `LightClientUpdate` for the referenced block  header because it cannot produce the necessary signatures from the sync committee.
  - If the block header is unknown, consult the provided slot number. If there is another block at this slot, start the challenge process. It should not be possible for the sync committee to have signed a different block at the same height. If this was a missed slot and there is not enough evidence that the sync committee haven't signed such a block, wait for a new light client update. The operator will have only rare opportunities to post such ambiguous updates and the resulting delays would allow another operator to take over the bridge (these policies will be provided as a general mechanism for ensuring than an operator who goes out of business can be replaced).
  - If the `LightClientUpdate` matches the commitment, verify that the rest of the provided data is authentic.

Upon implementation, Blockdaemon commits to operate, promote, and maintain the developed bridges in all official networks. Furthermore, Blockdaemon commits to create and promote several innovative projects backed by academic research, on top of the provided trustless oracle:

1) Blockchain Migration

   The goal of the investigation is to build a proof of concept migrating data from a dApp on Ethereum 2.0 to a EVM-compatible chain in a trustless way (validating state migration via the oracle). This is project would be the first introducing trustless , automatic state migration across chains.

2) Integration with [Hyperledger Cactus](This is project would be the first introducing trustless , automatic state migration across chains).

   Hyperledger Cactus is the leading open-source, enterprise-grade interoperability project. Cactus aims to promote integration between enterprise systems and different blockchains. Cactus also provides support for developing infrastructure that integrates with Ethereum, and the developed oracle and bridge, including but not limited to: operators, relayers, and products on top of the bridge/oracle. The goal of this investigation is to diminish the entry barrier to enterprises wanting to use Ethereum 2.0. We can extend the current Ethereum connector to support Ethereum 2.0, and facilitate the integration with the developed bridges.

### 2023

Goals:

* Create a custom Ethereum testnet, using a `hash_tree_root` based on hash function such as Pedersen Hash or Poseidon that is more friendly to zero-knowledge circuits.

* Implement the light client sync algorithm as a zero-knowledge circuit and explore the feasibility of creating and verifying single shot light client updates skipping multiple periods at once.

* Validate our use cases and produce academic research based on them.

## Collaboration and Reporting

All development will take place in this Github repository and all communications related to the project will be carried out in a public [Telegram group](https://t.me/ProjectDendrETH). The team will publish weekly reports intended for the EF research team and other technical audiences, summarizing the most imporant findings and developments.

Upon important project milestones, Blockdaemon will publish a series of explainer articles promoting the project to the general public.

