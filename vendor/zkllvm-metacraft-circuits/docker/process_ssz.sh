#!/bin/sh

for v in $(find /DendrETH -iname "*ssz_snappy" | grep "weigh_justification_and_finalization_test")
do 
    echo $v
    rm -f ${v}.json
    /go/bin/zcli pretty capella BeaconState ssz_snappy:${v} ${v}.json
    slot=$(cat ${v}.json | grep "\"slot\"" | grep -E "^[ ]{2}[\"a-z]{1,}" | awk '/slot/ { gsub(/[",]/,"",$2); print $2}')
    sed -i '$ d' ${v}.json
    echo "," >> ${v}.json
    current_gindex=$(( 303104 + (($slot / 32) * 32) % 8192 ))
    previous_gindex=$(( 303104 + ((($slot / 32) * 32) - 32) % 8192 ))
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices 34 | grep witn | tac | \
    awk 'BEGIN{printf("\"slot_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices 50 | grep witn | tac | \
    awk 'BEGIN{printf("\"previous_justified_checkpoint_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices 51 | grep witn | tac | \
    awk 'BEGIN{printf("\"current_justified_checkpoint_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices 49 | grep witn | tac | \
    awk 'BEGIN{printf("\"justification_bits_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices 52 | grep witn | tac | \
    awk 'BEGIN{printf("\"finalized_checkpoint_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices $current_gindex | grep leaf | \
    awk '{printf("\"current_epoch_start_slot_root_in_block_roots\":\"%s\"", $3)}' >> ${v}.json
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices $current_gindex | grep witn | tac | \
    awk 'BEGIN{printf("\"current_epoch_start_slot_root_in_block_roots_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices $previous_gindex | grep leaf | \
    awk '{printf("\"previous_epoch_start_slot_root_in_block_roots\":\"%s\"", $3)}' >> ${v}.json
    echo "," >> ${v}.json
    /go/bin/zcli proof capella BeaconState ssz_snappy:${v} --gindices $previous_gindex | grep witn | tac | \
    awk 'BEGIN{printf("\"previous_epoch_start_slot_root_in_block_roots_proof\": [ ");}{current=prev;prev=$3;if(current != "") printf("\"%s\", ", current)}END{printf("\"%s\" ]", prev)}' >> $v.json;
    echo "," >> ${v}.json
    /go/bin/zcli root capella BeaconState ssz_snappy:${v} | awk '{printf("\"beacon_state_root\": \"%s\"", $1)}' >> ${v}.json
    echo "" >> ${v}.json
    echo "}" >> ${v}.json
done


