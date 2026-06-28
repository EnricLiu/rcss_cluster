#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="FRA-UNIted"
mc_parse_player_args "$@"
cd "$BASE/agent/bin" || exit 1
opt="-host $host -port $port -t_name $teamname -agent_conf ../conf/agent.conf -formations_conf ../conf/formations.conf"
if [ -n "$log_dir" ]; then
  opt="$opt -log_dir $log_dir -log_lev 0"
elif [ -n "$debugopt" ]; then
  opt="$opt -log_lev 0"
fi
if [ "$goalie" = true ]; then
  exec ./FRA-UNIted_Agent -goalie $opt
else
  exec ./FRA-UNIted_Agent $opt
fi
