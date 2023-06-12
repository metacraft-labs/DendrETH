pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";

template hashAggregatedKey(){
  signal input aggregatedKey[384];
  signal output out[256];

  component hash = Sha256(512);

  for(var i = 0; i < 384; i++) {
    hash.in[i] <== aggregatedKey[i];
  }

  for(var i = 384; i < 512; i++) {
    hash.in[i] <== 0;
  }

  for(var i = 0; i < 256; i++) {
  out[i] <== hash.out[i];
  }


}

component main = hashAggregatedKey();
