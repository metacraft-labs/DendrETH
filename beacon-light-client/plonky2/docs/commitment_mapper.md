# Optimizing ZK Proofs for Validator Balances

## Objective

Our goal is to confirm the total of a certain collection of validator balances on-chain, aiming to diminish the amount of required on-chain Merkle proofs. Our current strategy utilizes one zk-proof validation on-chain to ensure that a specific validator set exhibits a certain balance total for a designated beacon state root.

The primary obstacle encountered is the lack of zk-friendliness in the SHA-256 hash function, which is presently employed in our Merkle proofs. The performance deficiency due to the significant computational burden of the SHA-256 makes it unsuitable for our requirements. To illustrate, a single Merkle proof for a validator demands about 43 hashes, whereas the balance tree calls for 39 hashes.

## Suggested Approach

### Handling Validators

We propose to create a commitment mapping from the validators' root (a SHA-256 Merkle tree comprising all validators) to a Poseidon root of validators. This process would involve generating a proof that a given SHA-256 Merkle tree of validators matches a corresponding Poseidon Merkle tree of validators. Given the sheer number of validators, this tree will be formed using recursive proofs. Since only a small fraction of validators changes per epoch, we can economically update and reuse the proofs. The Poseidon hash function, being more zk-friendly, could then be used to validate that a specific validator belongs to the tree in a more cost-effective manner.

### Dealing with Balances

The complexity with balances lies in their tendency to alter with each epoch. Consequently, adopting a similar approach as the one for validators would not be feasible. This is due to the need for recalculating the proofs for all balances from the ground up for each epoch.
