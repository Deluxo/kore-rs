let
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/nixos-24.11.tar.gz")) {};
in
pkgs.callPackage (
  {
    mkShell,
    cargo,
    rustc,
    pkg-config,
    gtk4,
    libsoup_3,
    glib,
    pango,
    gdk-pixbuf,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      cargo
      rustc
      pkg-config
    ];
    buildInputs = [
      gtk4
      libsoup_3
      glib
      pango
      gdk-pixbuf
    ];
    LIBRARY_PATH = "${pkgs.gdk-pixbuf}/lib";
  }
) { }
