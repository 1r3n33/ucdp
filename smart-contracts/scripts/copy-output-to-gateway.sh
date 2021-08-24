#!/usr/bin/bash

set -e

# set current dir to this script dir
cd "${0%/*}"

# copy the .abi part of the truffle output to gateway resources directory
cat ../build/contracts/Ucdp.json | jq .abi > ../../gateway/res/Ucdp.abi.json
