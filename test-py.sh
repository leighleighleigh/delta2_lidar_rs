#!/usr/bin/env bash

# Need to move to a different folder, otherwise it tries to import in here.
x=$(pwd)

# Adds the nix build to pythonpath, opens python
export PYTHONPATH="${PYTHONPATH}:${x}/result/lib/python3.10/site-packages/"

pushd $(mktemp -d)

cp $x/examples/stream-rerun.py .
python3 stream-rerun.py

popd
