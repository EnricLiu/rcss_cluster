#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="YuShan2024"
coach="$BASE/YuShan2024_coach"
coach_conf="$BASE/data/coach.conf"
team_graphic="--use_team_graphic on"
extra_coach_opt=""
mc_parse_coach_args "$@"
mc_prepend_ld_path "$BASE/data/lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_coach
