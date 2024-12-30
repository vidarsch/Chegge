#!/bin/bash

PID=$(ss -ltnp | grep ':8080' | awk '{print $6}' | cut -d',' -f2 | cut -d'=' -f2)

if [ -z "$PID" ]; then
  echo "No process found on port 8080."
else
  echo "Killing process with PID: $PID"
  # Kill the process
  kill -9 "$PID"
  
fi