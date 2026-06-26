#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="Oxsy"
player="$BASE/bin/oxsyplayer"
mc_parse_player_args "$@"
cd "$BASE" || exit 1
opt="-server_ip $host -player_port $port -team_name $teamname"
if [ "$goalie" = true ]; then
  exec "$player" $opt -goalie
else
  exec "$player" $opt
fi
