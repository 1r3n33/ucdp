#!/bin/sh

# Abort on any error (including if wait-for-it fails).
set -e

# Wait for kafka to be up, if we know where it is.
if [ -n "$UCDP_STREAM_KAFKA_BROKER" ]; then
  /ucdp/gateway/scripts/wait-for-it.sh -t 30 "$UCDP_STREAM_KAFKA_BROKER"
fi

# Run the main container command.
exec "$@"
