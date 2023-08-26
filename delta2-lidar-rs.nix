{ lib
, rustPlatform
, pkg-config
}:

rustPlatform.buildRustPackage {
  buildInputs = [ pkg-config ];
  nativeBuildInputs = [ pkg-config ];

  name = "delta2_lidar_rs";

  src = lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  outputs = [ "out" "dev" ];
}
