#!/usr/bin/env bash

# Slot: 4759061
# |               Status |   Count |
# | -------------------- | ------- |
# |  pending_initialized |     167 |
# |       pending_queued |    1154 |
# |       active_ongoing |  432276 |
# |       active_exiting |       0 |
# |       active_slashed |       6 |
# |     exited_unslashed |       0 |
# |       exited_slashed |      26 |
# |  withdrawal_possible |     860 |
# |      withdrawal_done |       0 |
# |               active |       0 |
# |              pending |       0 |
# |               exited |       0 |
# |           withdrawal |       0 |

SLOT=${1:-head}
BASE="http://127.0.0.1:5052"
STATUSES=(
    pending_initialized pending_queued active_ongoing active_exiting
    active_slashed exited_unslashed exited_slashed withdrawal_possible
    withdrawal_done active pending exited withdrawal
)

latest_slot()
{
    URL="$BASE/eth/v2/beacon/blocks/$SLOT"
    echo $(curl $URL 2>/dev/null | jq .data.message.slot) | sed 's/"//g'
}

validators_by_status()
{
    SLOT=$1
    STATUS=$2

    # Can't use $URL?status=$STATUS as nimbus apparently has a bug.
    URL="$BASE/eth/v1/beacon/states/$SLOT/validators"
    echo $(curl $URL 2>/dev/null | jq ".data | map(select(.status == \"$STATUS\")) | length")
}

main()
{
    [ $SLOT == head ] && SLOT=$(latest_slot)
    echo "Slot: $SLOT"
    echo "|               Status |   Count |"
    echo "| -------------------- | ------- |"
    for STATUS in ${STATUSES[@]}
    do
        printf "| %20s | %7d |\n" $STATUS $(validators_by_status $SLOT $STATUS)
    done
}

main
