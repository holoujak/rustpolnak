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

## Build & Run

```bash
$ dx serve
```

Start testing API server:
```bash
$ cd stubs
$ uv run fastapi dev
```

Run `rfid.py` to create virtual serials `stubs/dev/rfid0` and `stubs/dev/rfid1` and begin publishing TAGs:
```bash
$ cd stubs
$ uv run rfid.py
```

Run `rfiddump` to test RFID readers:
```
$ cargo run --bin rfiddump
$ cargo run --bin rfiddump /dev/ttyUSB0 /dev/ttyUSB1
```
