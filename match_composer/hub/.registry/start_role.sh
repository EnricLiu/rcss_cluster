#!/bin/sh

root=$1
role=$2
shift 2

host=localhost
unum=
goalie=false

while [ $# -gt 0 ]; do
  case "$1" in
    -h|--host)
      host=$2
      shift
      ;;
    -p|--port|-P|--coach-port|-t|--teamname|--log-dir)
      shift
      ;;
    -u|--unum)
      unum=$2
      shift
      ;;
    -g|--goalie)
      goalie=true
      ;;
    --debug|--offline-logging|--offline-client-mode|--debug-server-connect|--debug-server-logging)
      ;;
    --debug-*|--debug_*|--debug-server-host|--debug-server-port|--debug-log-ext|--fullstate)
      shift
      ;;
  esac
  shift
done

if [ -f "$root/start" ]; then
  start=$root/start
elif [ -f "$root/bin/start" ]; then
  start=$root/bin/start
else
  echo "No supported start script found under $root" >&2
  exit 1
fi

base_dir=$(dirname "$start")

for lib_dir in "$root/lib" "$base_dir/lib" "$root" "$base_dir"; do
  if [ -d "$lib_dir" ]; then
    if [ -z "$LD_LIBRARY_PATH" ]; then
      LD_LIBRARY_PATH=$lib_dir
    else
      LD_LIBRARY_PATH=$lib_dir:$LD_LIBRARY_PATH
    fi
  fi
done
export LD_LIBRARY_PATH

case "$role" in
  player)
    if [ "$goalie" = true ] || [ "$unum" = 1 ]; then
      num=1
    else
      num=${unum:-2}
    fi
    ;;
  coach)
    num=12
    ;;
  *)
    echo "Unsupported role: $role" >&2
    exit 1
    ;;
esac

echo "init ok."

exec /bin/sh "$start" "$host" "$base_dir" "$num"
