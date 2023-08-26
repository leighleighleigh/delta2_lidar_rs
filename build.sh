#!/usr/bin/env bash

# Experimental build of this mixed rust+python package using nix
# Thanks, hacker1024!

# build rust (library)
#nix-build -E '((import <nixpkgs> { }).callPackage ./delta2-lidar-rs.nix { }).dev'
#ls result-dev/include/delta2_lidar_rs

# build python
nix-build -E '(import <nixpkgs> { }).python3Packages.callPackage ./delta2-lidar-py.nix { }'
# cross-compile for aarch64
#nix-build -E '(import <nixpkgs> { }).pkgsCross.aarch64-multiplatform.python3Packages.callPackage ./delta2-lidar-py.nix { }'

#python setup.py bdist_wheel --plat-name x86_64-unknown-linux-gnu --py-limited-api=cp38
