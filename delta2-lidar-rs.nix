{ lib
, rustPlatform
, pkg-config
, systemd
, udev
}:

rustPlatform.buildRustPackage {
  buildInputs = [ pkg-config systemd udev ];
  nativeBuildInputs = [ pkg-config ];

  name = "delta2_lidar_rs";

  src = lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  outputs = [ "out" "dev" ];
}
