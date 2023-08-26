{ lib
, buildPythonPackage
, rustPlatform
, pkg-config
, cargo
, rustc
, setuptools-rust
, toml
}:

buildPythonPackage rec {
  name = "delta2_lidar";

  src = lib.cleanSource ./.;
  sourceRoot = "source/";

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };

  buildInputs = [ pkg-config ];

  nativeBuildInputs = [
    cargo
    rustPlatform.cargoSetupHook
    rustc
    setuptools-rust
    toml
    pkg-config
  ];


  # postPatch = ''
  #   chmod u+w ..
  #   ln -s ../Cargo.lock .
  # '';
}
