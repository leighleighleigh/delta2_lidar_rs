{ lib
, buildPythonPackage
, rustPlatform
, pkg-config
, systemd
, udev 
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

  buildInputs = [ pkg-config systemd udev ];

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
