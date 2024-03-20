#!/bin/sh

set -o errexit

#cargo build
if [ "_$OSTYPE" == "_msys" ] ; then
EXE_SUFFIX=".exe"
GREP_EXE_SUFFIX="\.exe"
else
EXE_SUFFIX=""
fi

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
kill_all
exit 0
