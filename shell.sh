#!/usr/bin/env bash

nix-shell -p python3 '(import <nixpkgs> { }).python3Packages.callPackage ./build.nix { }'
