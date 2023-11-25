{ pkgs ? import <nixpkgs> {} }:                                                    
let
  delta2-lidar = pkgs.python310Packages.callPackage ./derivation.nix {};
in
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
      delta2-lidar
      python3
    ];

  shellHook = ''
  export PYTHONPATH="''${PYTHONPATH}:${delta2-lidar}/lib/python3.10/site-packages/"
  '';

  #LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
}
