pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/poseidon.circom";

template VerifierPoseidon(pubInpCount, k) {
    signal input originator[2];
    signal input nextHeaderHashNum[2];
    signal input historicSyncCommitteeHashTreeRoot;
    signal input syncCommitteeHistoricParticipationIndex;

    // Verification Key
    signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
    signal input gamma2[2][2][k];
    signal input delta2[2][2][k];
    signal input IC[pubInpCount+1][2][k];
    signal input domain[256];

    signal output out;

    var negalfa1xbeta2_index = 6 * 2 * k;
    var gamma2_index =  2 * 2 * k;
    var delta2_index = 2 * 2 * k;
    var IC_index = (pubInpCount + 1) * 2 * k;

    var cummulative_index = 0;

    var commitment_size = 2 + 2 + 1 + negalfa1xbeta2_index + gamma2_index + delta2_index + IC_index;

    component commitment = HashTreeRootPoseidon(185); 

    for (var i = 0; i < 6; i++) { 
        for (var j = 0; j < 2; j++) {
            for (var q = 0; q < k; q++) {
                commitment.in[cummulative_index + i*2*k + j*k + q] <== negalfa1xbeta2[i][j][q];
            }
        }
    }

    cummulative_index += 6 * 2 * k;

    for (var i = 0; i < 2; i++) { 
        for (var j = 0; j < 2; j++) {
            for (var q = 0; q < k; q++) {
                commitment.in[cummulative_index + i*2*k + j*k + q] <== gamma2[i][j][q];
            }
        }
    }

    cummulative_index += 2 * 2 * k;

    for (var i = 0; i < 2; i++) { 
        for (var j = 0; j < 2; j++) {
            for (var q = 0; q < k; q++) {
                commitment.in[cummulative_index + i*2*k + j*k + q] <== delta2[i][j][q];
            }
        }
    }

    cummulative_index += 2 * 2 * k;

    for (var i = 0; i < pubInpCount + 1; i++) { 
        for (var j = 0; j < 2; j++) {
            for (var q = 0; q < k; q++) {
                commitment.in[cummulative_index + i*2*k + j*k + q] <== IC[i][j][q];
            }
        }
    }

    cummulative_index += (pubInpCount + 1)*2*k;

    for (var i = 0; i < 2; i++) {
        commitment.in[cummulative_index + i] <== originator[i];
    }
    
    cummulative_index += 2;

    for (var i = 0; i < 2; i++) {
        commitment.in[cummulative_index + i] <== nextHeaderHashNum[i];
    }

    cummulative_index += 2;

    for (var i = 0; i < 256; i++) {
        commitment.in[cummulative_index + i] <== domain[i];
    }

    cummulative_index += 256;

    commitment.in[cummulative_index] <== historicSyncCommitteeHashTreeRoot;

    cummulative_index += 1;

    commitment.in[cummulative_index] <== syncCommitteeHistoricParticipationIndex;

    out <== commitment.out;
}
