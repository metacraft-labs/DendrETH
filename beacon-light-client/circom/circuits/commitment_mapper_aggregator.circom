pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "hash_tree_root_pedersen.circom";
include "hash_tree_root.circom";

template CommitmentMapperAggregator(N) {
  var pubInpCount = 1;
  // BN254 facts
  var k = 6;

  signal input shaHashes[N][256];
  signal input pedersenHashes[N];

  // verification key
  signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input gamma2[2][2][k];
  signal input delta2[2][2][k];
  signal input IC[pubInpCount+1][2][k];

  signal input zeroOnFirst;

  // proof
  signal input negpa[N][2][k];
  signal input pb[N][2][2][k];
  signal input pc[N][2][k];

  signal output output_commitment;

  component hashTreeRootPedersen = HashTreeRootPedersen(N);
    component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    hashTreeRootPedersen.leaves[i] <== pedersenHashes[i];
    for (var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== shaHashes[i][j];
    }
  }

  component commitment = Pedersen(401);

  for(var i = 0; i < 256; i++) {
    commitment.in[i] <== hashTreeRoot.out[i];
  }

  commitment.in[256] <== hashTreeRootPedersen.out;

  for(var i = 0; i < 6; i++) {
    for(var j = 0; j<2;j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[257 + i * 12 + j * 6 + idx] <== zeroOnFirst * negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[329 + i * 12 + j * 6 + idx] <== zeroOnFirst * gamma2[i][j][idx];
        commitment.in[353 + i * 12 + j * 6 + idx] <== zeroOnFirst * delta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < pubInpCount + 1; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[377 + i * 12 + j * 6 + idx] <== zeroOnFirst * IC[i][j][idx];
      }
    }
  }

  output_commitment <== commitment.out[0];

  var firstGama2[2][2][6] = [
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

  var firstDelta2[2][2][6] =  [
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

  var firstNegalfa1xbeta2[6][2][6] = [
    [
      [
        4063420080633,
        6555003798509,
        3528875089017,
        5800537096256,
        8041381108016,
        203518374640
      ],
      [
        7676269984398,
        1145806392863,
        6738515895690,
        5144301275423,
        8547057760405,
        353834589854
      ]
    ],
    [
      [
        5712635615088,
        8763475698695,
        7480760495871,
        1630925336586,
        5902994417779,
        229051200835
      ],
      [
        1066113280330,
        5452941452156,
        130670027992,
        6364438679415,
        8227984268724,
        117895881848
      ]
    ],
    [
      [
        2720638156466,
        8183746692879,
        2805734624200,
        4541538633192,
        1476702149455,
        162434980571
      ],
      [
        4093955238700,
        4839352246179,
        5773319594517,
        5269728708172,
        8404179905859,
        4522318692
      ]
    ],
    [
      [
        7907150524416,
        8555524456643,
        2425990496019,
        5117607179458,
        886559720121,
        343845114320
      ],
      [
        3348806304058,
        5295378168489,
        5426585403009,
        4313512356362,
        2882006508456,
        312905790371
      ]
    ],
    [
      [
        6984987484510,
        4411212100320,
        517962775393,
        5578757090043,
        1344911245314,
        115782940661
      ],
      [
        4257694794763,
        5641455412912,
        2987387394488,
        6147130513016,
        8766894161060,
        7451503335
      ]
    ],
    [
      [
        3338043330865,
        3023333978926,
        4787719622265,
        3729967781503,
        2489094582823,
        396043239802
      ],
      [
        3390886416082,
        169102433935,
        2279828268438,
        1618451670976,
        7055320302964,
        48334526481
      ]
    ]
  ];

  var firstIC[2][2][6] = [
    [
      [
        5873542164103,
        2098934358288,
        8348997535046,
        6260821287013,
        1377192983638,
        166033587745
      ],
      [
        5482004592777,
        7680451724103,
        893995251219,
        5567166733382,
        7738779137723,
        114126095586
      ]
    ],
    [
      [
        2386502774126,
        154821628083,
        2120827065500,
        6700066818033,
        528338382096,
        149601566691
      ],
      [
        4184670564857,
        840823418264,
        759091980244,
        531294481298,
        7664383991370,
        377327298965
      ]
    ]
  ];

  component groth16Verifier[N];
  component prevCommitments[N];

  for(var index = 0; index < N; index++) {
    groth16Verifier[index] = verifyProof(pubInpCount);

    for (var i = 0;i < 6;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].negalfa1xbeta2[i][j][idx] <== zeroOnFirst * negalfa1xbeta2[i][j][idx] + (1 - zeroOnFirst) * firstNegalfa1xbeta2[i][j][idx];
        }
      }
    }

    for (var i = 0;i < 2;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].gamma2[i][j][idx] <== zeroOnFirst * gamma2[i][j][idx] + (1 - zeroOnFirst) * firstGama2[i][j][idx];
          groth16Verifier[index].delta2[i][j][idx] <== zeroOnFirst * delta2[i][j][idx] + (1 - zeroOnFirst) * firstDelta2[i][j][idx];
          groth16Verifier[index].pb[i][j][idx] <== pb[index][i][j][idx];
        }
      }
    }

    for (var i = 0;i < pubInpCount + 1;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].IC[i][j][idx] <== IC[i][j][idx];
        }
      }
    }

    for (var i = 0;i < 2;i++) {
      for (var idx = 0;idx < k;idx++) {
        groth16Verifier[index].negpa[i][idx] <== negpa[index][i][idx];
        groth16Verifier[index].pc[i][idx] <== pc[index][i][idx];
      }
    }

    prevCommitments[index] = Pedersen(401);

    for(var i = 0; i < 256; i++) {
      prevCommitments[index].in[i] <== shaHashes[index][i];
    }

    prevCommitments[index].in[256] <== pedersenHashes[index];

    for(var i = 0; i < 6; i++) {
      for(var j = 0; j<2;j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[257 + i * 12 + j * 6 + idx] <== zeroOnFirst * negalfa1xbeta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < 2; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[329 + i * 12 + j * 6 + idx] <== zeroOnFirst * gamma2[i][j][idx];
          prevCommitments[index].in[353 + i * 12 + j * 6 + idx] <== zeroOnFirst * delta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < pubInpCount + 1; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[377 + i * 12 + j * 6 + idx] <== zeroOnFirst * IC[i][j][idx];
        }
      }
    }
  }
}
