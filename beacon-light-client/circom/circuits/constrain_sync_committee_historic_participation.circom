pragma circom  2.1.5;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template updateSyncCommitteeHistoricParticipation(N, Periods) {
    signal input participationRateArray[Periods];
    signal input currentIndex;
    signal input bitmask[N];

    signal output out[Periods];

    var participationRate = 0;
    for (var i=0;i<N;i++) {
        participationRate += bitmask[i];
    }

    //Constrain
    signal isValidIndex <== LessThan(32)([currentIndex, Periods]);
    isValidIndex === 1;

    component isZero[Periods];
    for (var i=currentIndex;i<Periods;i++) {
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

    for (var i=currentIndex + 1;i<Periods;i++) {
        out[i] <== 0;
    }

}
