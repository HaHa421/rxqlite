#!/bin/sh

set -o errexit

if [ "_$OSTYPE" == "_msys" ] ; then
EXE_SUFFIX=".exe"
GREP_EXE_SUFFIX="\.exe"
else
EXE_SUFFIX=""
fi
#cargo build

kill_all() {
    
    if [ "_$OSTYPE" == "_msys" ] ; then
      SERVICE="rxqlited"
      if ps | grep "${SERVICE}" ; then 
        ps | grep "${SERVICE}" | awk '{print $1}' | xargs kill
      fi
      rm -r 127.0.0.1:*.db || echo "no db to clean"
    else
      SERVICE="rxqlited"
      if [ "$(uname)" = "Darwin" ]; then
          if pgrep -xq -- "${SERVICE}"; then
              pkill -f "${SERVICE}"
          fi
          rm -r 127.0.0.1:*.db || echo "no db to clean"
      else
          set +e # killall will error if finds no process to kill
          killall "${SERVICE}"
          set -e
      fi
    fi
}

rpc() {
    local uri=$1
    local body="$2"

    echo '---'" rpc(:$uri, $body)"

    {
        if [ ".$body" = "." ]; then
            time curl --silent "127.0.0.1:$uri"
        else
            time curl --silent "127.0.0.1:$uri" -H "Content-Type: application/json" -d "$body"
        fi
    } | {
        if type jq > /dev/null 2>&1; then
            jq
        else
            cat
        fi
    }

    echo
    echo
}

export RUST_LOG=trace
export RUST_BACKTRACE=full
bin=./target/release/rxqlited${EXE_SUFFIX}

echo "Killing all running rxqlited and cleaning up old data"

kill_all
sleep 1


echo "Start 3 uninitialized rxqlited servers..."

RUST_LOG="debug" ${bin} --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 2>&1 > n1.log &
PID1=$!
#sleep 1
echo "Server 1 started"

RUST_LOG="debug" ${bin} --id 2 --http-addr 127.0.0.1:21002 --rpc-addr 127.0.0.1:22002 > n2.log &
#sleep 1
echo "Server 2 started"

RUST_LOG="debug" ${bin} --id 3 --http-addr 127.0.0.1:21003 --rpc-addr 127.0.0.1:22003 > n3.log &

exit 0

