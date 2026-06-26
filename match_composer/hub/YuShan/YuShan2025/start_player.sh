#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$DIR/../../.registry/helios_common.sh"

BASE="$DIR"
teamname="YuShan2025"
player="$BASE/YuShan2025_Player"
player_conf="$BASE/data/player.conf"
config_dir="$BASE/data/formations-dt"
extra_player_opt=""
mc_parse_player_args "$@"
mc_prepend_ld_path "$BASE/data/lib"
cd "$BASE/." 2>/dev/null || cd "$BASE" || exit 1
mc_exec_librcsc_player
