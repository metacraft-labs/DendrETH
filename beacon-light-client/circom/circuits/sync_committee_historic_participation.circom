pragma circom  2.1.5;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template UpdateSyncCommitteeHistoricParticipation(N, PERIODS) {
    signal input participationRateArray[PERIODS];
    signal input currentIndex;
    signal input bitmask[N];

    signal output out[PERIODS];

    var participationRate = 0;
    for (var i=0;i<N;i++) {
        participationRate += bitmask[i];
    }

    //Constrain
    signal isValidIndex <== LessThan(32)([currentIndex, PERIODS]);
    isValidIndex === 1;

    component isZero[PERIODS];
    for (var i=currentIndex;i<PERIODS;i++) {
        isZero[i] = IsZero();
        isZero[i].in <== participationRateArray[i];
        isZero[i].out === 1;
    }

    for (var i=0;i<currentIndex;i++) {
        isZero[i] = IsZero();
        isZero[i].in <== participationRateArray[i];
        isZero[i].out === 0;
    }

    //Calc. new entry
    var bitmask_sum = 0;
    for (var i=0;i<N;i++) {
        bitmask_sum += bitmask[i];
    }

    // Assign
    for (var i=0;i<currentIndex;i++) {
        out[i] <== participationRateArray[i];
    }
    
    out[currentIndex] <== bitmask_sum;

    for (var i=currentIndex + 1;i<PERIODS;i++) {
        out[i] <== 0;
    }

}
