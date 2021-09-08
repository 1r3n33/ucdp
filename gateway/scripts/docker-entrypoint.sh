#!/bin/sh

# Abort on any error (including if wait-for-it fails).
set -e

# Wait for kafka, aerospike to be up, if we know where it is.
if [ -n "$UCDP_STREAM_KAFKA_BROKER" ]; then
  /ucdp/gateway/scripts/wait-for-it.sh -t 30 "$UCDP_STREAM_KAFKA_BROKER"
fi
if [ -n "$UCDP_CACHE_AEROSPIKE_HOST" ]; then
  /ucdp/gateway/scripts/wait-for-it.sh -t 30 "$UCDP_CACHE_AEROSPIKE_HOST"
fi

# Run the main container command.
exec "$@"
