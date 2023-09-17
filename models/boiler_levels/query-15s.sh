#!/usr/bin/env bash

set -euo pipefail

DB_PATH="../../gesha.db"

QUERY=$(
    cat <<SQL
SELECT *
FROM measurement
WHERE time >= __from__ AND time <= __to__
SQL
)

start_times=("1691509346385" "1691509421395" "1691509594569" "1691509893573" "1691510322250" "1691510894928" "1691511584562" "1691512411742" "1691513362647" "1691514406413")
end_times=("1691509421395" "1691509594569" "1691509893573" "1691510322250" "1691510894928" "1691511584562" "1691512411742" "1691513362647" "1691514406413" "1691515570391")

for i in "${!start_times[@]}"; do
    LOCAL_QUERY=$(sed -e "s/__from__/${start_times[$i]}/" -e "s/__to__/${end_times[$i]}/" <<<"$QUERY")
    sqlite3 $DB_PATH -header -csv "$LOCAL_QUERY" >"measurement-history-heat-level-15s-$(expr $i + 1).csv"
done
