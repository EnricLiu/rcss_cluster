#!/bin/sh

mc_need_value() {
  if [ "$2" -lt 2 ]; then
    echo "Missing value for $1" >&2
    exit 1
  fi
}

mc_prepend_ld_path() {
  for lib_dir in "$@"; do
    if [ -d "$lib_dir" ]; then
      if [ -z "$LD_LIBRARY_PATH" ]; then
        LD_LIBRARY_PATH=$lib_dir
      else
        LD_LIBRARY_PATH=$lib_dir:$LD_LIBRARY_PATH
      fi
    fi
  done
  export LD_LIBRARY_PATH
}

mc_default_debug_endpoint() {
  if [ -z "$debug_server_host" ]; then
    debug_server_host=$host
  fi
  if [ -z "$debug_server_port" ]; then
    debug_server_port=`expr "$port" + 32`
  fi
}

mc_parse_player_args() {
  host=${host:-localhost}
  port=${port:-6000}
  teamname=${teamname:-HELIOS}
  unum=${unum:-}
  goalie=${goalie:-false}
  debug_server_host=${debug_server_host:-}
  debug_server_port=${debug_server_port:-}
  debugopt=${debugopt:-}
  debug_opt=${debug_opt:-}
  offline_logging=${offline_logging:-}
  offline_mode=${offline_mode:-}
  fullstateopt=${fullstateopt:-}
  log_dir=${log_dir:-}

  while [ $# -gt 0 ]; do
    case "$1" in
      --help)
        echo "Usage: $0 -h HOST -p PORT -t TEAM -u UNUM [-g] [--debug --log-dir DIR]" >&2
        exit 0
        ;;
      -h|--host)
        mc_need_value "$1" "$#"
        host=$2
        shift 2
        ;;
      -p|--port)
        mc_need_value "$1" "$#"
        port=$2
        shift 2
        ;;
      -t|--teamname)
        mc_need_value "$1" "$#"
        teamname=$2
        shift 2
        ;;
      -u|--unum)
        mc_need_value "$1" "$#"
        unum=$2
        shift 2
        ;;
      -g|--goalie)
        goalie=true
        shift
        ;;
      -f|--formation)
        mc_need_value "$1" "$#"
        config_dir=$2
        shift 2
        ;;
      --offline-logging)
        offline_logging="--offline_logging"
        shift
        ;;
      --offline-client-mode)
        offline_mode=on
        shift
        ;;
      --debug)
        debugopt="$debugopt --debug"
        shift
        ;;
      --debug-start-time)
        mc_need_value "$1" "$#"
        debug_opt="$debug_opt --debug_start_time $2"
        shift 2
        ;;
      --debug-end-time)
        mc_need_value "$1" "$#"
        debug_opt="$debug_opt --debug_end_time $2"
        shift 2
        ;;
      --debug-server-connect)
        debugopt="$debugopt --debug_server_connect"
        shift
        ;;
      --debug-server-host)
        mc_need_value "$1" "$#"
        debug_server_host=$2
        shift 2
        ;;
      --debug-server-port)
        mc_need_value "$1" "$#"
        debug_server_port=$2
        shift 2
        ;;
      --debug-server-logging)
        debugopt="$debugopt --debug_server_logging"
        shift
        ;;
      --log-dir)
        mc_need_value "$1" "$#"
        log_dir=$2
        debugopt="$debugopt --log_dir $2"
        shift 2
        ;;
      --debug-log-ext)
        mc_need_value "$1" "$#"
        debugopt="$debugopt --debug_log_ext $2"
        shift 2
        ;;
      --fullstate)
        mc_need_value "$1" "$#"
        case "$2" in
          ignore)
            fullstateopt="--use_fullstate false --debug_fullstate false"
            ;;
          reference)
            fullstateopt="--use_fullstate false --debug_fullstate true"
            ;;
          override)
            fullstateopt="--use_fullstate true --debug_fullstate true"
            ;;
          *)
            echo "Invalid --fullstate value: $2" >&2
            exit 1
            ;;
        esac
        shift 2
        ;;
      --debug_*|--debug-*)
        debug_opt="$debug_opt $1"
        shift
        ;;
      *)
        echo "Invalid player option: $1" >&2
        exit 1
        ;;
    esac
  done

  mc_default_debug_endpoint
}

mc_parse_coach_args() {
  host=${host:-localhost}
  port=${port:-6002}
  teamname=${teamname:-HELIOS}
  debug_server_host=${debug_server_host:-}
  debug_server_port=${debug_server_port:-}
  debugopt=${debugopt:-}
  debug_opt=${debug_opt:-}
  offline_logging=${offline_logging:-}
  log_dir=${log_dir:-}
  team_graphic=${team_graphic:-}

  while [ $# -gt 0 ]; do
    case "$1" in
      --help)
        echo "Usage: $0 -h HOST -p PORT -t TEAM [--debug --log-dir DIR]" >&2
        exit 0
        ;;
      -h|--host)
        mc_need_value "$1" "$#"
        host=$2
        shift 2
        ;;
      -p|--port|-P|--coach-port)
        mc_need_value "$1" "$#"
        port=$2
        shift 2
        ;;
      -t|--teamname)
        mc_need_value "$1" "$#"
        teamname=$2
        shift 2
        ;;
      --coach-config)
        mc_need_value "$1" "$#"
        coach_conf=$2
        shift 2
        ;;
      --team-graphic)
        mc_need_value "$1" "$#"
        team_graphic="--use_team_graphic on --team_graphic_file $2"
        shift 2
        ;;
      --offline-logging)
        offline_logging="--offline_logging"
        shift
        ;;
      --debug)
        debugopt="$debugopt --debug"
        shift
        ;;
      --debug-start-time)
        mc_need_value "$1" "$#"
        debug_opt="$debug_opt --debug_start_time $2"
        shift 2
        ;;
      --debug-end-time)
        mc_need_value "$1" "$#"
        debug_opt="$debug_opt --debug_end_time $2"
        shift 2
        ;;
      --debug-server-connect)
        debugopt="$debugopt --debug_server_connect"
        shift
        ;;
      --debug-server-host)
        mc_need_value "$1" "$#"
        debug_server_host=$2
        shift 2
        ;;
      --debug-server-port)
        mc_need_value "$1" "$#"
        debug_server_port=$2
        shift 2
        ;;
      --debug-server-logging)
        debugopt="$debugopt --debug_server_logging"
        shift
        ;;
      --log-dir)
        mc_need_value "$1" "$#"
        log_dir=$2
        debugopt="$debugopt --log_dir $2"
        shift 2
        ;;
      --debug-log-ext)
        mc_need_value "$1" "$#"
        debugopt="$debugopt --debug_log_ext $2"
        shift 2
        ;;
      --debug_*|--debug-*)
        debug_opt="$debug_opt $1"
        shift
        ;;
      *)
        echo "Invalid coach option: $1" >&2
        exit 1
        ;;
    esac
  done

  mc_default_debug_endpoint
}

mc_ping_host() {
  ping -c 1 "$host" >/dev/null 2>&1 || true
}

mc_exec_librcsc_player() {
  opt="--player-config $player_conf --config_dir $config_dir"
  opt="$opt -h $host -p $port -t $teamname"
  opt="$opt $fullstateopt"
  opt="$opt --debug_server_host $debug_server_host --debug_server_port $debug_server_port"
  opt="$opt $offline_logging $debugopt $debug_opt $extra_player_opt"

  offline_number=
  if [ -n "$offline_mode" ] && [ -n "$unum" ]; then
    offline_number="--offline_client_number $unum"
  fi

  mc_ping_host
  if [ "$goalie" = true ]; then
    exec "$player" $opt -g $offline_number
  else
    exec "$player" $opt $offline_number
  fi
}

mc_exec_librcsc_coach() {
  opt="--coach-config $coach_conf"
  opt="$opt -h $host -p $port -t $teamname"
  opt="$opt $team_graphic"
  opt="$opt --debug_server_host $debug_server_host --debug_server_port $debug_server_port"
  opt="$opt $offline_logging $debugopt $debug_opt $extra_coach_opt"

  mc_ping_host
  exec "$coach" $opt
}
