# pkgs refers to the pinned nixpkgs
{ pkgs, lib, config, ... }:

let
  openGlDeps = with pkgs; [
    mesa
    libGL
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
  openGlPaths = lib.makeLibraryPath openGlDeps;
in
{
  languages.rust.enable = true;
  languages.python.enable = true;
  languages.python.uv.enable = true;
  packages = with pkgs; [
    dioxus-cli
    glib
    gtk3
    webkitgtk_4_1
    xdotool
  ] ++ openGlDeps;
  env = {
    LD_LIBRARY_PATH = openGlPaths;
    WEBKIT_DISABLE_COMPOSITING_MODE = 1;
  };
  # enterTest = ''
  #   cargo build --profile release
  #   cargo test --profile release
  # '';
  processes = {
    api.exec = "uv --directory stubs run fastapi dev";
    rfid.exec = "uv --directory stubs run rfid.py";
  };
  scripts = {
    test-app.exec = ''
      cargo build --profile release
      cargo test --profile release
    '';
  };
}
