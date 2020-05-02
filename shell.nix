{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "nexromancers-hacksaw-shell";
  nativeBuildInputs = [
    pkgs.pkgconfig
    pkgs.python3
  ];
  buildInputs = [
    pkgs.cargo
    pkgs.clippy
    pkgs.rustc
    pkgs.rustfmt
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
  ];
}
