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

  # format is now 'pyproject', default is 'setuptools'
  format = "pyproject";

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
