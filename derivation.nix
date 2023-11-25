{ pkgs ? import <nixpkgs> {} }:                                                    
pkgs.python310Packages.buildPythonPackage rec {
  pname = "delta2-lidar";
  version = "0.1.0";

  src = pkgs.lib.cleanSource ./.;
  sourceRoot = "source/"; # the base folder of the repo

  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    rustPlatform.cargoSetupHook
    python310Packages.setuptools-rust
    python310Packages.toml
    cargo
    rustc
  ];

  pythonImportsCheck = [
    "delta2_lidar"
  ];

  format = "setuptools";

  # postPatch = ''
  #   chmod u+w ..
  #   ln -s ../Cargo.lock .
  # '';
}
