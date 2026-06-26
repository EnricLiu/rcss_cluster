#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR/bin"
teamname="RoboCIn"
coach="$BASE/src/robocin_coach"
coach_conf="$BASE/src/coach.conf"
team_graphic="--use_team_graphic on --team_graphic_file $BASE/src/team_logo.xpm"
extra_coach_opt=""
export PYTHONHOME="$BASE/lib"
mc_parse_coach_args "$@"
mc_prepend_ld_path "$BASE/lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_coach
