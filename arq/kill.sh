#!/bin/bash
TARGETPID=$(ps -o pid:1,command:1 -u $(whoami) | grep -E "[0-9]+ ./target/debug/arq" | head -n 1 | cut -d ' ' -f 1)

if [ -z $TARGETPID ];
then
  echo "No arq process found.."
else 
  kill $TARGETPID && echo "Killed pid: $TARGETPID"
fi
