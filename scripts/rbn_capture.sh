#!/bin/bash

if [ $# -ne 4 ]
  then
    echo "usage: rbn_capture.sh TELNET_HOST TELNET_PORT CALLSIGN CAPTURE_SECONDS"
    exit 1
fi
 
TELNET_HOST=$1
TELNET_PORT=$2
CALLSIGN=$3
DELAY=$4
LOG_FILE="rbn_capture$(date +%Y%m%d_%H%M%S).log"
 
echo "Starting Telnet session. Logging to $LOG_FILE..."
 
(
  echo "$CALLSIGN"
  sleep $DELAY
  echo "exit"
) | telnet "$TELNET_HOST" "$TELNET_PORT" | tee "$LOG_FILE"  # Log and display
 
echo "Session complete. Log saved to $LOG_FILE"
