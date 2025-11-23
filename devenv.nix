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
  name = "rustpolnak";
  git-hooks.hooks = {
    # various
    end-of-file-fixer.enable = true;
    trim-trailing-whitespace.enable = true;
    check-yaml.enable = true;
    # rust
    rustfmt.enable = true;
    cargo-check.enable = true;
    # piton
    ruff-format.enable = true;
    ruff.enable = true;

  };
  languages.rust.enable = true;
  languages.python.enable = true;
  languages.python.uv.enable = true;
  packages = with pkgs; [
    dioxus-cli
    glib
    gtk3
    git
    tk
    webkitgtk_4_1
    xdotool
    socat
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
    dx.exec = "dx serve";
    api.exec = "uv --directory stubs run fastapi dev";
    rfid.exec = "uv --directory stubs run rfid.py";
    rfiddump.exec = "cargo run --bin rfiddump";
  };
  scripts = {
    test-app.exec = ''
      cargo build --profile release
      cargo test --profile release
    '';
  };
}


