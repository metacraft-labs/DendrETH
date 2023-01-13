pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/pedersen.circom";
include "validators_hash_tree_root.circom";
include "hash_tree_root_pedersen.circom";
include "compress.circom";

template CommitmentMapper(N) {
  var J = 2;
  var K = 7;

  signal input points[N][J][K];
  signal input withdrawCredentials[N][256];

  signal input effectiveBalance[N][256];
  signal input slashed[N];

  signal input activationEligibilityEpoch[N];
  signal input activationEpoch[N];

  signal input exitEpoch[N];
  signal input withdrawableEpoch[N][256];

  signal input zero[N];

  signal output output_commitment;

  component validatorsHashTreeRoot = ValidatorsHashTreeRoot(N);
  component pedersenHashTreeRoot = HashTreeRootPedersen(N);
  component pedersen[N];
  component compress[N];

  for (var i = 0; i < N; i++) {
    compress[i] = Compress();

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        compress[i].point[j][k] <== points[i][j][k];
      }
    }

    for(var j = 0; j < 384; j++) {
      validatorsHashTreeRoot.pubkeys[i][j] <== compress[i].bits[j];
    }

    validatorsHashTreeRoot.zero[i] <== zero[i];

    validatorsHashTreeRoot.activationEligibilityEpoch[i] <== activationEligibilityEpoch[i];
    validatorsHashTreeRoot.activationEpoch[i] <== activationEpoch[i];
    validatorsHashTreeRoot.exitEpoch[i] <== exitEpoch[i];
    validatorsHashTreeRoot.slashed[i] <== slashed[i];

    for(var j = 0; j < 256; j++) {
      validatorsHashTreeRoot.withdrawCredentials[i][j] <== withdrawCredentials[i][j];
      validatorsHashTreeRoot.effectiveBalance[i][j] <== effectiveBalance[i][j];
      validatorsHashTreeRoot.withdrawableEpoch[i][j] <== withdrawableEpoch[i][j];
    }

    pedersen[i] = Pedersen(18);

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        pedersen[i].in[j * 7 + k] <== points[i][j][k];
      }
    }

    pedersen[i].in[14] <== activationEligibilityEpoch[i];
    pedersen[i].in[15] <== activationEpoch[i];
    pedersen[i].in[16] <== exitEpoch[i];
    pedersen[i].in[17] <== slashed[i];

    pedersenHashTreeRoot.leaves[i] <== pedersen[i].out[0] * zero[i];
  }

  component commitment = Pedersen(257);

  for(var i = 0; i < 256; i++) {
    commitment.in[i] <== validatorsHashTreeRoot.out[i];
  }

  commitment.in[256] <== pedersenHashTreeRoot.out;

  output_commitment <== commitment.out[0];
}
