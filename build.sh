#!/usr/bin/env bash

# Experimental build of this mixed rust+python package using nix
# Thanks, hacker1024!
#nix-build -E '(import <nixpkgs> { }).python3Packages.callPackage ./build.nix { }'
# cross-compile for aarch64
#nix-build -E '(import <nixpkgs> { }).pkgsCross.aarch64-multiplatform.python3Packages.callPackage ./delta2-lidar-py.nix { }'

rm -rf build 
rm -rf ./py-src/*.egg-info
rm -rf dist
# build into ./build/ folder
python setup.py build -b ./build/
# run stubgen in the build folder
built=$(realpath ./build/lib*)
stubgen -o $built -p delta2_lidar --search-path="$build"

# wheel-up the build
python3 setup.py bdist_wheel -k --skip-build 

#pip install --upgrade --force-reinstall ./dist/delta2_lidar-0.1.0-cp310-cp310-linux_x86_64.whl 


