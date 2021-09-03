#!/bin/sh

# Abort on any error (including if wait-for-it fails).
set -e

# Start ganache
npx ganache-cli -d -m "ostrich van ladder motor youth please jacket art amused feel blush filter ripple awesome drip olive dutch luxury excess network husband aim cruise borrow" -h 0.0.0.0 -p 8545 -i 7777 &
# Wait for ganache
/ucdp/smart-contracts/scripts/wait-for-it.sh -t 30 0.0.0.0:8545

# Deploy contracts
npx truffle migrate --reset --compile-all --network docker --describe-json

# ucdp contract address must be 0x81F34DC2C089AF4e11881af04399a7e722feA6F4

# loop
while true; do sleep 10; done;
