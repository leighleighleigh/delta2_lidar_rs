{ lib
, buildPythonPackage
, rustPlatform
, cargo
, rustc
, setuptools-rust
, toml
}:

buildPythonPackage rec {
  name = "delta2_lidar_py";

  src = lib.cleanSource ./.;
  sourceRoot = "source/";

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    cargo
    rustPlatform.cargoSetupHook
    rustc
    setuptools-rust
    toml
  ];

  postPatch = ''
    chmod u+w ..
    ln -s ../Cargo.lock .
  '';
}
