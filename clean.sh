#!/usr/bin/env bash
rm result-dev
nix-collect-garbage --delete-older-than 1h
