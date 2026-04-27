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
g_ip="localhost"
g_port=50051
diff_g_port="false"
gp20="false"
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
   echo "      --debug-server-host HOST specifies debug server host (default: host)"
   echo "      --debug-server-port PORT specifies debug server port (default: port + 32)"
   echo "      --debug-server-logging   writes debug server log (default: off)"
   echo "      --log-dir DIRECTORY      specifies debug log directory (default: /tmp)"
   echo "      --debug-log-ext EXT      specifies debug log file extension (default: .log)"
   echo "      --g-ip GRPC IP           specifies grpc IP (default: localhost)"
   echo "      --g-port GRPC PORT       specifies grpc port (default: 50051)"
   echo "      --diff-g-port            specifies different grpc port for each player"
   echo "      --gp20                   add 20 to GRPC port if team runs on right side") 1>&2
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
      gp20="true"
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
coachopt="${coachopt} --g-ip ${g_ip}"
coachopt="${coachopt} --g-port ${g_port}"

if [ "${diff_g_port}" = "true" ]; then
  coachopt="${coachopt} --diff-g-port"
fi
if [ "${gp20}" = "true" ]; then
  coachopt="${coachopt} --gp20"
fi

ping -c 1 "$host" >/dev/null 2>&1

exec $coach ${coachopt}

