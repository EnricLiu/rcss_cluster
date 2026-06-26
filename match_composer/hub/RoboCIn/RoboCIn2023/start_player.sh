#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR"
teamname="RoboCIn"
player="$BASE/src/robocin_player"
player_conf="$BASE/src/player.conf"
config_dir="$BASE/src/formations/standard"
extra_player_opt=""
export PYTHONHOME="$BASE/lib/PY_ENV"
mc_parse_player_args "$@"
mc_prepend_ld_path "$BASE/lib" "$BASE/lib/PY_ENV/lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_player
