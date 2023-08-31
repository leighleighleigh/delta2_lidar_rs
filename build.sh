#!/usr/bin/env bash

# Experimental build of this mixed rust+python package using nix
# Thanks, hacker1024!
#nix-build -E '(import <nixpkgs> { }).python3Packages.callPackage ./build.nix { }'
# cross-compile for aarch64
#nix-build -E '(import <nixpkgs> { }).pkgsCross.aarch64-multiplatform.python3Packages.callPackage ./delta2-lidar-py.nix { }'

# Build function
function build_target()
{
    SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    export CROSS_CONTAINER_ENGINE=podman

    CROSSTARGET="${1}"
    # Build for aarch64
    # CROSSTARGET="aarch64-unknown-linux-gnu"
    # Build for x86_64
    # CROSSTARGET="x86_64-unknown-linux-gnu"

    # Change plat-name depending on CROSSTARGET
    if [[ "${CROSSTARGET}" == "aarch64-unknown-linux-gnu" ]];
    then
        PLATNAME="manylinux2014_aarch64"
    elif [[ "${CROSSTARGET}" == "x86_64-unknown-linux-gnu" ]];
    then
        PLATNAME="manylinux2014_x86_64"
    else
        echo "Unknown CROSSTARGET: ${CROSSTARGET}"
        exit 1
    fi   

    export CARGO=cross
    export CARGO_BUILD_TARGET=${CROSSTARGET}
    export CROSS_CONFIG=${SCRIPT_DIR}/Cross.toml

    # Very important to clean, incase old crates for x86 are present
    cargo clean
    cross build --target $CROSSTARGET --release

    # Remove old python builds
    rm -rf build 
    rm -rf ./py-src/*.egg-info
    rm -rf dist

    # build into ./build/ folder
    python setup.py build -b ./build/

    # [OPTIONAL] run stubgen in the built folder
    # built=$(realpath ./build/lib*)
    # stubgen -o $built -p delta2_lidar --search-path="$build"

    # wheel-up the build for this target
    python3 setup.py bdist_wheel -k --skip-build --plat-name $PLATNAME --py-limited-api=cp38
}

# Check if this script is being sourced, or executed
# https://stackoverflow.com/a/28776166/736079
if [[ "${BASH_SOURCE[0]}" == "${0}" ]];
then
    # Not sourced, so execute native build
    native_arch=$(uname -m)
    build_target "${native_arch}-unknown-linux-gnu"
    # install!
    pip install --upgrade --force-reinstall ./dist/delta2_lidar-*_$(uname -m).whl 
fi



