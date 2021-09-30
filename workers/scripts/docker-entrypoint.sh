#!/bin/sh

# Abort on any error (including if wait-for-it fails).
set -e

# Wait for dependencies to be up, if we know where they are.
if [ -n "$UCDP_STREAM_KAFKA_BROKER" ]; then
  /app/workers/scripts/wait-for-it.sh -t 30 "$UCDP_STREAM_KAFKA_BROKER"
fi

# Run the main container command.
exec "$@"
