#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="FRA-UNIted"
mc_parse_coach_args "$@"
cd "$BASE/coach/bin" || exit 1
opt="-server_9.4 0 -host $host -port $port -conf ../conf/coach.conf -teamName $teamname"
exec ./FRA-UNIted_Coach $opt
