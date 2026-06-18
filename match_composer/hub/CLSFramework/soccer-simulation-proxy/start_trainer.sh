#!/bin/sh

LIBPATH=/root/local/lib
if [ x"$LIBPATH" != x ]; then
  if [ x"$LD_LIBRARY_PATH" = x ]; then
    LD_LIBRARY_PATH=$LIBPATH
  else
    LD_LIBRARY_PATH=$LIBPATH:$LD_LIBRARY_PATH
  fi
  export LD_LIBRARY_PATH
fi

DIR=`dirname $0`

trainer="${DIR}/sample_trainer"
teamname="TRAINER_MODE"
host="localhost"
port=6001
rpc_host="localhost"
rpc_port=50051
rpc_port_step="false"
rpc_add_20_to_port_for_right="false"
rpc_type="grpc"
#config="${DIR}/player.conf"
#config_dir="${DIR}/formations-dt"

debugopt=""

usage()
{
  (echo "Usage: $0 [options]"
   echo "Possible options are:"
   echo "      --help                print this"
   echo "  -h, --host HOST           specifies server host"
   echo "  -p, --port PORT              specifies server coach port (default: 6002)"
   echo "      --debug                  writes debug log (default: off)"
   echo "      --log-dir DIRECTORY      specifies debug log directory (default: /tmp)"
   echo "  --rpc-host RPC host          specifies rpc host (default: localhost)"
   echo "  --rpc-port RPC PORT          specifies rpc port (default: 50051)"
   echo "  --rpc-port-step              specifies different rpc port for each player (default: false)"
   echo "  --rpc-add-20-to-port-for-right                    add 20 to RPC Port if team run on right side (default: false)"
   echo "  --rpc-type                   type of rpc framework (default: thrift) or grpc"
   echo "  -t, --teamname TEAMNAME   specifies team name") 1>&2
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
      host=$2
      shift 1
      ;;
    -p|--port|-P)
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
      teamname=$2
      shift 1
      ;;
    --debug)
      debugopt="${debugopt} --offline_logging --debug --debug_server_connect"
      ;;
    --log-dir)
      if [ $# -lt 2 ]; then
        usage
        exit 1
      fi
      debugopt="${debugopt} --log_dir ${2}"
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
      usage
      exit 1
      ;;
  esac

  shift 1
done

OPT="-h ${host} -p ${port} -t ${teamname}"
#OPT="${OPT} --player-config ${config} --config_dir ${config_dir}"
OPT="${OPT} ${debugopt}"
opt="${opt} --rpc-host ${rpc_host}"
opt="${opt} --rpc-port ${rpc_port}"
opt="${opt} --rpc-type ${rpc_type}"
if [ "${rpc_port_step}" = "true" ]; then
  opt="${opt} --rpc-port-step"
fi
if [ "${rpc_add_20_to_port_for_right}" = "true" ]; then
  opt="${opt} --rpc-add-20-to-port-for-right"
fi
#if [ $number -gt 0 ]; then
#  $player ${OPT} -g &
#  $sleepprog $goaliesleep
#fi

#for (( i=2; i<=${number}; i=$i+1 )) ; do

#done
trainer_opt="${trainer_opt} --rpc-host ${rpc_host}"
trainer_opt="${trainer_opt} --rpc-port ${rpc_port}"
trainer_opt="${trainer_opt} --rpc-type ${rpc_type}"

if [ "${rpc_port_step}" = "true" ]; then
  trainer_opt="${trainer_opt} --rpc-port-step"
fi
if [ "${rpc_add_20_to_port_for_right}" = "true" ]; then
  trainer_opt="${trainer_opt} --rpc-add-20-to-port-for-right"
fi

exec $trainer "$trainer_opt"
