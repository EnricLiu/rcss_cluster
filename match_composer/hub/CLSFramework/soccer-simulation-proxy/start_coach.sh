#!/bin/sh

echo "******************************************************************"
echo " HELIOS base"
echo " Created by Hidehisa Akiyama and Hiroki Shimora"
echo " Copyright 2000-2007.  Hidehisa Akiyama"
echo " Copyright 2007-2011.  Hidehisa Akiyama and Hiroki Shimora"
echo " All rights reserved."
echo "******************************************************************"

DIR=`dirname $0`

coach="${DIR}/sample_coach"
coach_conf="${DIR}/coach.conf"
teamname="SSP"
host="localhost"
port=6002
rpc_host="localhost"
rpc_port=50051
rpc_port_step="false"
rpc_add_20_to_port_for_right="false"
rpc_type="grpc"
debug_server_host=""
debug_server_port=""
team_graphic=""

debugopt=""
debug_opt=""
offline_logging=""

usage()
{
  (echo "Usage: $0 [options]"
   echo "Available options:"
   echo "      --help                   prints this"
   echo "  -h, --host HOST              specifies server host (default: localhost)"
   echo "  -p, --port PORT              specifies server coach port (default: 6002)"
   echo "  -P, --coach-port PORT        alias of --port"
   echo "  -t, --teamname TEAMNAME      specifies team name (default: SSP)"
   echo "      --coach-config FILE      specifies coach config file (default: ./coach.conf if it exists)"
   echo "      --team-graphic FILE      specifies the team graphic xpm file"
   echo "      --offline-logging        writes offline client log (default: off)"
   echo "      --debug                  writes debug log (default: off)"
   echo "      --debug_DEBUG_CATEGORY   writes DEBUG_CATEGORY to debug log"
   echo "      --debug-start-time TIME  the start time for recording debug log (default: -1)"
   echo "      --debug-end-time TIME    the end time for recording debug log (default: 99999999)"
   echo "      --debug-server-connect   connects to the debug server (default: off)"
   echo "      --debug-server-host HOST specifies debug server host (default: localhost)"
   echo "      --debug-server-port PORT specifies debug server port (default: 6032)"
   echo "      --debug-server-logging   writes debug server log (default: off)"
   echo "      --log-dir DIRECTORY      specifies debug log directory (default: /tmp)"
   echo "      --debug-log-ext EXT      specifies debug log file extension (default: .log)"
   echo "      --rpc-host RPC host      specifies rpc host (default: localhost)"
   echo "      --rpc-port RPC PORT      specifies rpc port (default: 50051)"
   echo "  --rpc-port-step              specifies different rpc port for each player (default: false)"
   echo "  --rpc-add-20-to-port-for-right                    add 20 to RPC Port if team run on right side (default: false)"
   echo "  --rpc-type                   type of rpc framework (default: thrift) or grpc"
   echo "                               FULLSTATE_TYPE is one of [ignore|reference|override].") 1>&2
}

while [ $# -gt 0 ]
do
  case $1 in
    --help)
      usage
      exit 0
      ;;

    -h|--host)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      host="${2}"
      shift 1
      ;;

    -p|--port|-P|--coach-port)
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

    --coach-config)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      coach_conf="${2}"
      shift 1
      ;;

    --team-graphic)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      team_graphic="--use_team_graphic on --team_graphic_file ${2}"
      shift 1
      ;;

    --offline-logging)
      offline_logging="--offline_logging"
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

    --rpc-host)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      rpc_host="${2}"
      shift 1
      ;;

    --rpc-port)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      rpc_port="${2}"
      shift 1
      ;;

    --rpc-port-step)
      rpc_port_step="true"
      ;;

    --rpc-add-20-to-port-for-right)
      rpc_add_20_to_port_for_right="true"
      ;;

    --rpc-type)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      rpc_type="${2}"
      shift 1
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

if [ X"${debug_server_host}" = X'' ]; then
  debug_server_host="${host}"
fi

if [ X"${debug_server_port}" = X'' ]; then
  debug_server_port=`expr ${port} + 32`
fi

coachopt=""
if [ -f "${coach_conf}" ]; then
  coachopt="--coach-config ${coach_conf}"
fi

coachopt="${coachopt} -h ${host} -p ${port} -t ${teamname}"
coachopt="${coachopt} ${team_graphic}"
coachopt="${coachopt} --debug_server_host ${debug_server_host}"
coachopt="${coachopt} --debug_server_port ${debug_server_port}"
coachopt="${coachopt} ${offline_logging}"
coachopt="${coachopt} ${debugopt}"
coachopt="${coachopt} ${debug_opt}"
coachopt="${coachopt} --rpc-host ${rpc_host}"
coachopt="${coachopt} --rpc-port ${rpc_port}"
coachopt="${coachopt} --rpc-type ${rpc_type}"

if [ "${rpc_port_step}" = "true" ]; then
  coachopt="${coachopt} --rpc-port-step"
fi
if [ "${rpc_add_20_to_port_for_right}" = "true" ]; then
  coachopt="${coachopt} --rpc-add-20-to-port-for-right"
fi

ping -c 1 "$host" >/dev/null 2>&1

exec $coach ${coachopt}

