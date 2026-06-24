#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$DIR/../../.registry/start_role.sh" "$DIR" coach "$@"
