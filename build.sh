#!/usr/bin/env bash

# Experimental build of this mixed rust+python package using nix
# Thanks, hacker1024!
nix-build -E '(import <nixpkgs> { }).python3Packages.callPackage ./delta2-lidar.nix { }'
