{ lib
, buildPythonPackage
, rustPlatform
, pythonOlder
, pkg-config
, setuptools
, setuptools-rust
, cargo
, rustc
, toml
}:

buildPythonPackage rec {
  pname = "delta2-lidar";
  version = "0.1.0";

  src = lib.cleanSource ./.;
  sourceRoot = "source/"; # the base folder of the repo

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };

  disabled = pythonOlder "3.8";

  nativeBuildInputs = [
    rustPlatform.cargoSetupHook
    pkg-config
    setuptools-rust
    cargo
    rustc
    toml
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
