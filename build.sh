#!/usr/bin/env bash

# Experimental build of this mixed rust+python package using nix
# Thanks, hacker1024!

# build rust (library)
#nix-build -E '((import <nixpkgs> { }).callPackage ./delta2-lidar-rs.nix { }).dev'
#ls result-dev/include/delta2_lidar_rs

# build python
nix-build -E '(import <nixpkgs> { }).python3Packages.callPackage ./delta2-lidar-py.nix { }'