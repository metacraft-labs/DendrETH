pragma circom 2.0.3;

include "numbersTo256Bits.circom";
include "hash_tree_root_beacon_header.circom";
include "output_commitment.circom";
include "hash_two.circom";
include "ssz_num.circom";
include "is_valid_merkle_branch.circom";
include "compute_domain.circom";
include "compute_signing_root.circom";
include "hash_to_field.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../vendor/circom-pairing/circuits/bls_signature.circom";

template FullClient() {
  var J = 2;
  var K = 7;
  signal input prevHeaderHashNum[2];
  signal input nextHeaderHashNum[2];

  component prevHeaderHash = NumbersTo256Bits();
  prevHeaderHash.first <== prevHeaderHashNum[0];
  prevHeaderHash.second <== prevHeaderHashNum[1];

  component nextHeaderHash = NumbersTo256Bits();
  nextHeaderHash.first <== nextHeaderHashNum[0];
  nextHeaderHash.second <== nextHeaderHashNum[1];

  // Should be hardcoded
  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal input slot;
  signal input proposer_index[256];
  signal input parent_root[256];
  signal input state_root[256];
  signal input body_root[256];

  signal input participantsSum;

  signal input numberOfValidators;
  signal input validatorsHash[256];

  signal input aggregatedKey[J][K];
  signal input validatorsBranch[5][256];

  // proof
  signal input negpa[2][6];
  signal input pb[2][2][6];
  signal input pc[2][6];

  signal input signature[2][2][K];

  component hash_tree_root_beacon = HashTreeRootBeaconHeader();

  component sszSlot = SSZNum(64);
  sszSlot.in <== slot;

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.slot[i] <== sszSlot.out[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.proposer_index[i] <== proposer_index[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.parent_root[i] <== parent_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.state_root[i] <== state_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.body_root[i] <== body_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.blockHash[i] === prevHeaderHash.out[i];
  }

  component commitment = OutputCommitment();

  commitment.currentEpoch <== slot / 32;
  commitment.participantsCount <== participantsSum;

  for(var i = 0; i < 256; i++) {
    commitment.hash[i] <== validatorsHash[i];
  }

  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      commitment.aggregatedKey[j][k] <== aggregatedKey[j][k];
    }
  }

  var negalfa1xbeta2[6][2][6] = [[[4063420080633, 6555003798509, 3528875089017, 5800537096256, 8041381108016, 203518374640], [7676269984398, 1145806392863, 6738515895690, 5144301275423, 8547057760405, 353834589854]], [[5712635615088, 8763475698695, 7480760495871, 1630925336586, 5902994417779, 229051200835], [1066113280330, 5452941452156, 130670027992, 6364438679415, 8227984268724, 117895881848]], [[2720638156466, 8183746692879, 2805734624200, 4541538633192, 1476702149455, 162434980571], [4093955238700, 4839352246179, 5773319594517, 5269728708172, 8404179905859, 4522318692]], [[7907150524416, 8555524456643, 2425990496019, 5117607179458, 886559720121, 343845114320], [3348806304058, 5295378168489, 5426585403009, 4313512356362, 2882006508456, 312905790371]], [[6984987484510, 4411212100320, 517962775393, 5578757090043, 1344911245314, 115782940661], [4257694794763, 5641455412912, 2987387394488, 6147130513016, 8766894161060, 7451503335]], [[3338043330865, 3023333978926, 4787719622265, 3729967781503, 2489094582823, 396043239802], [3390886416082, 169102433935, 2279828268438, 1618451670976, 7055320302964, 48334526481]]];

  var IC[2][2][6] = [[[2687064965962, 1262180070285, 3303251428028, 5272518547750, 456267978848, 363818299712], [6106953335802, 1390718626615, 7123978759627, 8587103750562, 2664683834221, 180048284494]], [[4350585007697, 5347381060664, 8693748120009, 7482128752468, 1351332976055, 200360903728], [1226991641673, 7304545383675, 2148898618993, 2379680186242, 449519966892, 103834876090]]];
  var gamma2[2][2][6] = [
    [
      [
        5896345417453,
        4240670514135,
        6172078461917,
        219834884668,
        2138480846496,
        206187650596
      ],
      [
        6286472319682,
        5759053266064,
        8549822680278,
        8639745994386,
        912741836299,
        219532437284
      ]
    ],
    [
      [
        4404069170602,
        525855202521,
        8311963231281,
        825823174727,
        854139906743,
        161342114743
      ],
      [
        3147424765787,
        7086132606363,
        7632907980226,
        5320198199754,
        6592898451945,
        77528801456
      ]
    ]
  ];

  var delta2[2][2][6] = [
    [
      [
        5896345417453,
        4240670514135,
        6172078461917,
        219834884668,
        2138480846496,
        206187650596
      ],
      [
        6286472319682,
        5759053266064,
        8549822680278,
        8639745994386,
        912741836299,
        219532437284
      ]
    ],
    [
      [
        4404069170602,
        525855202521,
        8311963231281,
        825823174727,
        854139906743,
        161342114743
      ],
      [
        3147424765787,
        7086132606363,
        7632907980226,
        5320198199754,
        6592898451945,
        77528801456
      ]
    ]
  ];

  for(var i = 0; i < 6; i++) {
    for(var j = 0; j<2;j++) {
      for(var idx = 0; idx < 6; idx++) {
        commitment.negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < 6; idx++) {
        commitment.gamma2[i][j][idx] <== gamma2[i][j][idx];
        commitment.delta2[i][j][idx] <== delta2[i][j][idx];
        commitment.IC[i][j][idx] <== IC[i][j][idx];
      }
    }
  }

  component groth16Verifier = verifyProof(1);

  for (var i = 0;i < 6;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        groth16Verifier.negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        groth16Verifier.gamma2[i][j][idx] <== gamma2[i][j][idx];
        groth16Verifier.delta2[i][j][idx] <== delta2[i][j][idx];
        groth16Verifier.IC[i][j][idx] <== IC[i][j][idx];
        groth16Verifier.pb[i][j][idx] <== pb[i][j][idx];
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var idx = 0;idx < 6;idx++) {
      groth16Verifier.negpa[i][idx] <== negpa[i][idx];
      groth16Verifier.pc[i][idx] <== pc[i][idx];
    }
  }

  groth16Verifier.pubInput[0] <== commitment.out;
  groth16Verifier.out === 1;

  component hashTwo = HashTwo();

  component sszNum = SSZNum(64);

  sszNum.in <== slot / 32;

  for(var i = 0; i < 256; i++) {
    hashTwo.in[0][i] <== sszNum.out[i];
    hashTwo.in[1][i] <== validatorsHash[i];
  }

  component isValidMerkleBranch = IsValidMerkleBranch(5);

  for(var i = 0; i < 5; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranch.branch[i][j] <== validatorsBranch[i][j];
    }
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranch.leaf[i] <== hashTwo.out[i];
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranch.root[i] <== state_root[i];
  }

  isValidMerkleBranch.index <== 43;

  isValidMerkleBranch.out === 1;

  component computeDomain = ComputeDomain();

  for(var i = 0; i < 32; i++) {
    computeDomain.fork_version[i] <== fork_version[i];
    computeDomain.DOMAIN_SYNC_COMMITTEE[i] <== DOMAIN_SYNC_COMMITTEE[i];
  }

  for(var i = 0; i < 256; i++) {
    computeDomain.GENESIS_VALIDATORS_ROOT[i] <== GENESIS_VALIDATORS_ROOT[i];
  }

  component computeSigningRoot = ComputeSigningRoot();

  for(var i = 0; i < 256; i++) {
    computeSigningRoot.headerHash[i] <== nextHeaderHash.out[i];
  }

  for(var i = 0; i < 256; i++) {
    computeSigningRoot.domain[i] <== computeDomain.domain[i];
  }

  component hashToField = HashToField();

  for(var i = 0; i < 256; i++) {
    hashToField.in[i] <== computeSigningRoot.signing_root[i];
  }

  component verify = CoreVerifyPubkeyG1(55, K);

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < K; k++) {
      verify.pubkey[j][k] <== aggregatedKey[j][k];
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        verify.signature[i][j][k] <== signature[i][j][k];
        verify.hash[i][j][k] <== hashToField.out[i][j][k];
      }
    }
  }
}
