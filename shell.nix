{ pkgs ? import <nixpkgs> {} }:                                                    
let
  delta2-lidar = pkgs.python310Packages.callPackage ./derivation.nix {};
in
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
      delta2-lidar
      python3
      python310Packages.pip
    ];

  shellHook = '''';

  LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
}
