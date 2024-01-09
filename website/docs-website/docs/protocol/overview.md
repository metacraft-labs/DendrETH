---
title: Overview
---

# What is DendrETH? 

### DendrETH is a smart contract implementation of the light client sync protocol.

The DendrETH project implements the beacon chain light client syncing algorithm in the form of a smart contract for multiple targeted blockchains. For EVM-based blockchains, we build upon prior research by 0xPARC, Darwinia, Alex Stokes and the Nimbus team to deliver the first end-to-end implementation capable of syncing the entire Mainnet history since Altair. Our current Solidity contract leverages a Circom zero-knowledge circuit to verify the BLS signatures of the Ethereum 2.0 validators and all of the syncing protocol rules.

The low bandwidth requirements of the Ethereum light client sync protocol create an unique opportunity for third-party blockchains to host smart contracts that implement a fully functional Ethereum light client.
Such a light client would be able to provide frequently updated information regarding recently finalised beacon chain block headers which in turn can be used to authenticate any portion of the Ethereum consensus or execution layer states through a chain of merkle proofs (or other commitment schemes in the future).

This would allow other smart contracts in the third-party blockchain to react to events happening in the Ethereum ecosystem, a functionality often described as a cross-blockchain bridge. Numerous bridge designs have already been proposed and implemented in the past, but a common shortcoming is that they rely on trusted oracles in their foundations, creating a systemic risk that the bridge may be compromised if the oracles are compromised.

A bridge based on the light client sync protocol will authenticate all data through the signatures of the Ethereum validators participating in the sync committees, significantly reducing the required level of trust in the bridge operator and limiting the potential for attacks.

The DendrETH project aims to develop highly efficient Ethereum light client implementations in the form of smart contracts for multiple third-party blockchains such as Solana, Polkadot, Cosmos, Cardano, Avalanche, Elrond, EOS, NEAR, Tezos and any EVM-compatible chain.

Our syncing and verification engine can be ported to a wide range of environments, from micro-controllers to web browsers.

Since the base circuit is able to verify complete header-to-header transitions, we also provide a recursive variant that can be used by any Ethereum client to implement one-shot syncing capabilities similar to the ones available in the Mina protocol (please see our analysis regarding the limitations of this approach).

The project is currently aiming to implement a zero-knowledge circuit capable of proving the Casper finality conditions by processing the attestation messages of the entire Ethereum validator set.