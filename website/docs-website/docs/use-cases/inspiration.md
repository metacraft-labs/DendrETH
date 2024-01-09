---
title: More Inspiration
---

TODO: add Oracles, Goveranance, Staking etc...

1. Proof of Total Locked Value
Given a merkle accumulator containing all public keys of the validators created by the liquid staking protocol, provide a proof for the sum of the balances of all validators at the last finalized epoch.

2. Proof of Total Rewards Potential
Given a merkle accumulator containing a set of validator public keys (this scales from a single validator to the entire set of validators created by the liquid staking protocol), provide a proof that indicates the maximum number of rewards that the set of validators were eligible for within the canonical finalized history. This takes into account the block proposal duties and the sync committee duties of the validators.

Since the maximum possible profit from a block cannot be determined without knowledge of all attestations that were broadcast in the network, we assume that the potential reward is equal to a running average of the last N blocks leading up to the slot of the proposal.

Similarly, since the maximum MEV profit from a block cannot be known, we can model it as a public input for the circuit which can be set by the liquid staking protocol (and potentially managed dynamically).

Please note that the circuit can verify the presence of a transaction within the block that distributes the MEV rewards to the liquid staking protocol, but unfortunately this doesn’t rule out the possibility that the proposer was paid some additional sum by the builder out-of-band. Some of these difficulties in the tracking of MEV rewards are likely to be resolved in the planned Proposer-Builder Separation upgrade of Ethereum that will enshrine the MEV distribution within the base protocol.

3. Proof of Poor Validator Performance
Given a Proof 2 obtained for a particular validator as described above, provide a proof that the validator has earned less than a target percentage of the maximum rewards (e.g. 90%). Such a proof can be used to penalize or evict particular operators from the protocol. The circuit compares all recorded withdrawals of the validator (while taking into account any deposit top-ups) to the total rewards potential to determine whether the validator is meeting the target performance.

4. Proof of Slashing
Certain protocols can benefit from explicit proofs for slashing events, which can be provided either as regular SSZ merkle tree proofs or as zero-knowledge components that can be embedded in larger proofs.

5. Proof of Deposit
A common mitigation for the well-known validator deposit front-running attack is the execution of 1 ETH initial deposit before the rest of the 32 ETH are committed. Proofs for verifying that a validator with a particular index and the desired withdrawal credentials has been created can be provided both as regular SSZ merkle proofs and as zero-knowledge circuit components.