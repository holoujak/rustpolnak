# rustpolnak

## Prerequisite

### Debian/Ubuntu
```sh
$ apt-get update
$ apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Archlinux
```sh
sudo pacman -S --needed \
    webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    xdotool \
    openssl \
    libayatana-appindicator \
    librsvg \
    socat
```

### Common
```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
$ rustup toolchain install stable
$ curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
$ cargo binstall dioxus-cli
```

## Configuration
Configuration is loaded from the following paths, in order of priority:
1. command line argument
2. `rustpolnak.toml` in current working directory
3. `$XDG_CONFIG_HOME/rustpolnak.toml`
4. `~/.config/rustpolnak.toml`

## Set up your dev environment
Prerequisites - have `nix` and  `devenv` in your system:

`$ nix shell nixpkgs#devenv` to get `devenv` into your `$PATH` in a temporary shell

OR

`$ nix profile add nixpkgs#devenv` to install imperatively to your system

`$ devenv shell` to enter the dev environment

Optional: automatically enter the environment via `direnv`
- Have [`direnv`](https://direnv.net/docs/installation.html) in your system
- On first `cd` into this repository, you may need to call `direnv allow` to whitelist this path
- On subsequent `cd` into this repository, `direnv` should pick up automatically and drop you into a `devenv shell`

## Build & Run

Run the app:
`$ devenv up dx`
`$ cargo run`

Start testing API server:
`$ devenv up api`

Create virtual serials `stubs/dev/rfid0` and `stubs/dev/rfid1` and begin publishing TAGs:
`$ devenv up rfid`

Run `rfiddump` to test RFID readers:
`$ devenv up rfiddump`

Run both API and virtual RFID:
`$ devenv up api rfid`


Run `rfiddump` to test RFID readers:
```
$ cargo run --bin rfiddump
$ cargo run --bin rfiddump /dev/ttyUSB0 /dev/ttyUSB1
```
