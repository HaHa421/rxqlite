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
      rm -r data-* || echo "no db to clean"
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

if ls 127.0.0.1:*.db
then
    rm -r 127.0.0.1:*.db || echo "no db to clean"
fi

echo "Start 3 uninitialized rxqlited servers..."

RUST_LOG="debug" ${bin} --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 2>&1 > n1.log &
PID1=$!
sleep 1
echo "Server 1 started"

RUST_LOG="debug" ${bin} --id 2 --http-addr 127.0.0.1:21002 --rpc-addr 127.0.0.1:22002 > n2.log &
sleep 1
echo "Server 2 started"

RUST_LOG="debug" ${bin} --id 3 --http-addr 127.0.0.1:21003 --rpc-addr 127.0.0.1:22003 > n3.log &
sleep 1
echo "Server 3 started"
sleep 1

echo "Initialize server 1 as a single-node cluster"
sleep 2
echo
rpc 21001/cluster/init '{}'

echo "Server 1 is a leader now"

sleep 2

echo "Get metrics from the leader"
sleep 2
echo
rpc 21001/cluster/metrics
sleep 1


echo "Adding node 2 and node 3 as learners, to receive log from leader node 1"

sleep 1
echo
rpc 21001/cluster/add-learner       '[2, "127.0.0.1:21002", "127.0.0.1:22002"]'
echo "Node 2 added as learner"
sleep 1
echo
rpc 21001/cluster/add-learner       '[3, "127.0.0.1:21003", "127.0.0.1:22003"]'
echo "Node 3 added as learner"
sleep 1

echo "Get metrics from the leader, after adding 2 learners"
sleep 2
echo
rpc 21001/cluster/metrics
sleep 1

echo "Changing membership from [1] to 3 nodes cluster: [1, 2, 3]"
echo
rpc 21001/cluster/change-membership '[1, 2, 3]'
sleep 1
echo "Membership changed"
sleep 1

echo "Get metrics from the leader again"
sleep 1
echo
rpc 21001/cluster/metrics
sleep 1



