#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR"
teamname="The8"
coach="$BASE/The8_coach"
coach_conf="$BASE/coach.conf"
team_graphic=""
extra_coach_opt=""
mc_parse_coach_args "$@"
mc_prepend_ld_path "$BASE/Lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_coach
