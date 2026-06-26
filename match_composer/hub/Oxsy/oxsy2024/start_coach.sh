#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="Oxsy"
coach="$BASE/bin/oxsycoach"
mc_parse_coach_args "$@"
cd "$BASE" || exit 1
opt="-server_ip $host -coach_port $port -team_name $teamname"
exec "$coach" $opt
