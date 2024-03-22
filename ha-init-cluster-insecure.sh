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
    else
      SERVICE="rxqlited"
      if [ "$(uname)" = "Darwin" ]; then
          if pgrep -xq -- "${SERVICE}"; then
              pkill -f "${SERVICE}"
          fi
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
            time curl --silent -k "https://127.0.0.1:$uri"
        else
            time curl --silent -k "https://127.0.0.1:$uri" -H "Content-Type: application/json" -d "$body"
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

generate_self_signed_certificate()
{
  mkdir -p certs-test
  #openssl genrsa -des3 -out certs-test/rxqlited.key 1024
  openssl genrsa -out certs-test/rxqlited.key 2048

  openssl req -x509 -new -nodes -key certs-test/rxqlited.key -sha256 -days 10000 -out certs-test/rxqlited.pem -subj "//C=Ha/ST=Ha/L=Ha/O=Test/OU=Test/CN=Ha"

  
  
}

export RUST_LOG=trace
export RUST_BACKTRACE=full
bin=./target/release/rxqlited${EXE_SUFFIX}

echo "Killing all running rxqlited and cleaning up old data"

kill_all
sleep 1

if ls data-*
then
    rm -r data-* || echo "no db to clean"
fi

if ls certs-test 
then
    rm -r certs-test || echo "no test certificates to clean"
fi

generate_self_signed_certificate



#RUST_LOG="debug" ${bin} --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 --leader --cert-path certs-test/rxqlited.pem --key-path  certs-test/rxqlited.key  --accept-invalid-certificates true  2>&1 > n1.log &
#PID1=$!
#sleep 1

#curl --silent -k "https://127.0.0.1:21001/cluster/metrics"

#exit 0


echo "Start 3 uninitialized rxqlited servers..."

RUST_LOG=debug ${bin} --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 --member "2;127.0.0.1:21002;127.0.0.1:22002" --member "3;127.0.0.1:21003;127.0.0.1:22003" --leader  --cert-path certs-test/rxqlited.pem --key-path  certs-test/rxqlited.key  --accept-invalid-certificates  2>&1 > n1.log &
PID1=$!
sleep 1
echo "Server 1 started as leader"
#exit 0
RUST_LOG=debug ${bin} --id 2 --http-addr 127.0.0.1:21002 --rpc-addr 127.0.0.1:22002 --cert-path certs-test/rxqlited.pem --key-path  certs-test/rxqlited.key  --accept-invalid-certificates > n2.log &
sleep 1
echo "Server 2 started as learner"

RUST_LOG=debug ${bin} --id 3 --http-addr 127.0.0.1:21003 --rpc-addr 127.0.0.1:22003 --cert-path certs-test/rxqlited.pem --key-path  certs-test/rxqlited.key  --accept-invalid-certificates > n3.log &
sleep 1
echo "Server 3 started as learner"
sleep 1


echo "Get metrics from the leader"
sleep 2
echo
rpc 21001/cluster/metrics
sleep 1


echo "Get metrics from the leader again"
sleep 1
echo
rpc 21001/cluster/metrics
sleep 1

