#!/bin/sh

echo "******************************************************************"
echo " HELIOS base"
echo " Created by Hidehisa Akiyama and Hiroki Shimora"
echo " Copyright 2000-2007.  Hidehisa Akiyama"
echo " Copyright 2007-2011.  Hidehisa Akiyama and Hiroki Shimora"
echo " All rights reserved."
echo "******************************************************************"


DIR=`dirname $0`

player="${DIR}/sample_player"
teamname="HELIOS_base"
host="localhost"
port=6000
g_ip="localhost"
g_port=50051
diff_g_port="false"
gp20="false"
debug_server_host=""
debug_server_port=""

player_conf="${DIR}/player.conf"
config_dir="${DIR}/formations-dt"

goalie_opt=""
unum=""
debugopt=""
debug_opt=""
offline_logging=""
offline_mode=""
fullstateopt=""

usage()
{
  (echo "Usage: $0 [options]"
   echo "Available options:"
   echo "      --help                   prints this"
   echo "  -h, --host HOST              specifies server host (default: localhost)"
   echo "  -p, --port PORT              specifies server port (default: 6000)"
   echo "  -t, --teamname TEAMNAME      specifies team name (default: HELIOS_base)"
   echo "  -u, --unum UNUM              specifies the uniform number (optional)"
   echo "  -g, --goalie                 starts as a goalie"
   echo "  -f, --formation DIR          specifies the formation directory"
   echo "  --offline-logging            writes offline client log (default: off)"
   echo "  --offline-client-mode        starts as an offline client (default: off)"
   echo "  --debug                      writes debug log (default: off)"
   echo "  --debug_DEBUG_CATEGORY       writes DEBUG_CATEGORY to debug log"
   echo "  --debug-start-time TIME      the start time for recording debug log (default: -1)"
   echo "  --debug-end-time TIME        the end time for recording debug log (default: 99999999)"
   echo "  --debug-server-connect       connects to the debug server (default: off)"
   echo "  --debug-server-host HOST     specifies debug server host (default: localhost)"
   echo "  --debug-server-port PORT     specifies debug server port (default: 6032)"
   echo "  --debug-server-logging       writes debug server log (default: off)"
   echo "  --log-dir DIRECTORY          specifies debug log directory (default: /tmp)"
   echo "  --debug-log-ext EXTENSION    specifies debug log file extension (default: .log)"
   echo "  --fullstate FULLSTATE_TYPE   specifies fullstate model handling [ignore|reference|override]") 1>&2
   echo "  --g-ip GRPC IP               specifies grpc IP (default: localhost)"
   echo "  --g-port GRPC PORT           specifies grpc port (default: 50051)"
}

while [ $# -gt 0 ]
do
  case $1 in

    --help)
      usage
      exit 0
      ;;
    --g-ip)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      g_ip="${2}"
      shift 1
      ;;
    --g-port)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      g_port="${2}"
      shift 1
      ;;
    --diff-g-port)
      diff_g_port="true"
      ;;
    --gp20)
      gp2="true"
      ;;
    -h|--host)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      host="${2}"
      shift 1
      ;;

    -p|--port)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      port="${2}"
      shift 1
      ;;

    -t|--teamname)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      teamname="${2}"
      shift 1
      ;;

    -u|--unum)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      unum="${2}"
      shift 1
      ;;

    -g|--goalie)
      goalie_opt="-g"
      ;;

    -f|--formation)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      config_dir="${2}"
      shift 1
      ;;

    --offline-logging)
      offline_logging="--offline_logging"
      ;;

    --offline-client-mode)
      offline_mode="on"
      ;;

    --debug)
      debugopt="${debugopt} --debug"
      ;;

    --debug_*)
      debug_opt="${debug_opt} ${1}"
      ;;

    --debug-start-time)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
	  debug_opt="${debug_opt} --debug_start_time ${2}"
	  shift 1
	  ;;

    --debug-end-time)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
	  debug_opt="${debug_opt} --debug_end_time ${2}"
	  shift 1
	  ;;

    --debug-server-connect)
      debugopt="${debugopt} --debug_server_connect"
      ;;

    --debug-server-host)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      debug_server_host="${2}"
      shift 1
      ;;

    --debug-server-port)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      debug_server_port="${2}"
      shift 1
      ;;

    --debug-server-logging)
      debugopt="${debugopt} --debug_server_logging"
      ;;

    --log-dir)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      debugopt="${debugopt} --log_dir ${2}"
      shift 1
      ;;

    --debug-log-ext)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      debugopt="${debugopt} --debug_log_ext ${2}"
      shift 1
      ;;

    --fullstate)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      fullstate_type="${2}"
      shift 1

      case "${fullstate_type}" in
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
          usage
          exit 1
          ;;
      esac
      ;;

    *)
      echo 1>&2
      echo "invalid option \"${1}\"." 1>&2
      echo 1>&2
      usage
      exit 1
      ;;
  esac

  shift 1
done

if  [ X"${offline_logging}" != X'' ]; then
  if  [ X"${offline_mode}" != X'' ]; then
    echo "'--offline-logging' and '--offline-mode' cannot be used simultaneously."
    exit 1
  fi
fi

if [ X"${debug_server_host}" = X'' ]; then
  debug_server_host="${host}"
fi

if [ X"${debug_server_port}" = X'' ]; then
  debug_server_port=`expr ${port} + 32`
fi

opt="--player-config ${player_conf} --config_dir ${config_dir}"
opt="${opt} -h ${host} -p ${port} -t ${teamname}"
opt="${opt} ${fullstateopt}"
opt="${opt} --debug_server_host ${debug_server_host}"
opt="${opt} --debug_server_port ${debug_server_port}"
opt="${opt} ${offline_logging}"
opt="${opt} ${debugopt}"
opt="${opt} ${debug_opt}"
opt="${opt} --g-ip ${g_ip}"
opt="${opt} --g-port ${g_port}"
if [ "${diff_g_port}" = "true" ]; then
  opt="${opt} --diff-g-port"
fi
if [ "${gp20}" = "true" ]; then
  opt="${opt} --gp20"
fi

offline_number=""
if  [ X"${offline_mode}" != X'' ] && [ X"${unum}" != X'' ]; then
  offline_number="--offline_client_number ${unum}"
fi

ping -c 1 $host >/dev/null 2>&1

if [ X"${goalie_opt}" != X'' ]; then
  exec $player ${opt} ${goalie_opt} ${offline_number}
else
  exec $player ${opt} ${offline_number}
fi
