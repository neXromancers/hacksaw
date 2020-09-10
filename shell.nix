{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "nexromancers-hacksaw-shell";
  nativeBuildInputs = [
    pkgs.cargo
    pkgs.clippy
    pkgs.pkgconfig
    pkgs.python3
    pkgs.rustc
    pkgs.rustfmt
  ];
  buildInputs = [
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
  ];
}
