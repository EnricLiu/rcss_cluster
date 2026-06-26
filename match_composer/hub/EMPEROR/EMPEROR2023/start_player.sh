#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR"
teamname="EMPEROR"
player="$BASE/sample_player"
player_conf="$BASE/player.conf"
config_dir="$BASE/formations-dt"
extra_player_opt=""
mc_parse_player_args "$@"
mc_prepend_ld_path "$BASE/lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_player
