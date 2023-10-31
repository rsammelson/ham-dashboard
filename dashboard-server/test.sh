#!/bin/bash

rr () {
    ops[0]="KD9MEQ"
    ops[1]="KD9YWS"
    ops[2]="KD9RPL"
    ops[3]="K9ETS"

    ops[4]="KD9MEQ"
    ops[5]="KD9YWS"
    ops[6]="KD9RPL"

    ops[7]="KD9MEQ"
    ops[8]="KD9YWS"
    ops[9]="KD9MEQ"
    ops[10]="KD9YWS"

    count=$(($RANDOM % 600))

    size=${#ops[@]}
    index=$(($RANDOM % $size))
    op=${ops[$index]}

    echo "$op $count"

    curl \
        'http://localhost:3000/' \
        -H 'Accept-Encoding: gzip, deflate, br' \
        -H 'Content-Type: application/json' \
        -H 'Accept: application/json' \
        -H 'Connection: keep-alive' \
        -H 'DNT: 1' \
        -H 'Origin: http://localhost:3000' \
        --data-binary "{\"query\":\"# Write your query or mutation here\nquery {\n  mostRecent(count: $count, isRun: false, operator: \\\"$op\\\") {\n    recvCallsign\n    sentCallsign\n    operator\n    timestamp\n    mode\n    freqRx\n    recvSignalReport\n    sentSignalReport\n    isRunQso\n    isWorthPoints\n    isClaimedQso\n    section\n    isMult1\n    isMult2\n    isMult3\n  }\n  entries {\n    recvCallsign\n  }\n}\"}" \
        --compressed \
        -s
}

for i in {0..4000}; do
    rr >/dev/null &
    sleep 0
done
