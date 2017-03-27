# UniPager

[![Build Status](https://img.shields.io/travis/rwth-afu/UniPager.svg?style=flat)](https://travis-ci.org/rwth-afu/UniPager)
[![GitHub issues](https://img.shields.io/github/issues/rwth-afu/UniPager.svg?style=flat)](https://github.com/rwth-afu/UniPager/issues)
[![GitHub release](https://img.shields.io/github/release/rwth-afu/UniPager.svg?style=flat)](https://github.com/rwth-afu/UniPager/releases)

Universal POCSAG transmitter controller written in Rust.

## Installation

### Automatic Installation

This script installs UniPager fully automatically on Debian/Raspbian systems.
It also uninstalls RustPager and migrates the old configuration file.

```bash
# Via HAMNET
curl http://db0sda.ampr.org/debian/install.sh -sSf | sh -s -- hamnet

# Via Internet
curl http://www.afu.rwth-aachen.de/debian/install.sh -sSf | sh -s -- internet
```

### Via HAMNET

Create the file `/etc/apt/sources.list.d/unipager.list` with the following content:

```
deb http://db0sda.ampr.org/debian unipager main
deb-src http://db0sda.ampr.org/debian unipager main
```

Then execute the following commands:

```bash
wget -O - http://ci.db0sda.ampr.org/debian/rwth-afu.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install unipager
```

### Via Internet

Create the file `/etc/apt/sources.list.d/unipager.list` with the following content:

```
deb http://www.afu.rwth-aachen.de/debian unipager main
deb-src http://www.afu.rwth-aachen.de/debian unipager main
```

Then execute the following commands:

```bash
wget -O - http://www.afu.rwth-aachen.de/debian/rwth-afu.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install unipager
```

## Compilation
Install rust:

```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

Now reboot OR log out to make the rust toolchain available.

Log in again and clone the source:

```bash
git clone https://github.com/rwth-afu/UniPager.git
```

If this command fails, you may need to install git and try again:

```bash
sudo apt-get install git
```

Start the build:

```bash
cd UniPager
cargo build --release
```
The compiled binary will be created at `./target/release/unipager`.

Be aware: Must be run with root privileges for GPIO access.

## Cross Compilation

Install rust:

```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

Install the GCC cross compiler:

```bash
sudo apt-get install -qq gcc-arm-linux-gnueabi # for soft float
sudo apt-get install -qq gcc-arm-linux-gnueabihf # for hard float
```

Define the target:

```bash
# ARMv6 with soft float
export TARGET="arm-unknown-linux-gnueabi"

# ARMv6 with hard float (e.g. Raspberry Pi 1)
export TARGET="arm-unknown-linux-gnueabihf"

# ARMv7 with hard float (e.g. Raspberry Pi 2 and 3)
export TARGET="armv7-unknown-linux-gnueabihf"
```

Install the cross-compiled rust libraries:

```bash
rustup target add $TARGET
```

Create the file `~/.cargo/config` with the following content:

```toml
[target.arm-unknown-linux-gnueabi]
linker = "arm-linux-gnueabi-gcc"

[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

Clone the source:

```bash
git clone https://github.com/rwth-afu/UniPager.git
```

Start the build:

```bash
cd UniPager
cargo build --target $TARGET --release 
```

The cross-compiled binary will be created at `./target/$TARGET/release/unipager`.

## Manual Installation

Move the UniPager binary to `/usr/local/bin/unipager`. Create the directory
`/var/lib/unipager`. Create the file `/etc/systemd/system/unipager.service` with
the following content:

```
[Unit]
Description=UniPager POCSAG transmitter controller
After=network.target

[Service]
ExecStart=/usr/local/bin/unipager
WorkingDirectory=/var/lib/unipager

[Install]
WantedBy=multi-user.target
```
Reload systemctl configuration with `sudo systemctl daemon-reload`.
To start UniPager enter `sudo systemctl start unipager`. To start UniPager
automatically after booting enter `sudo systemctl enable unipager`.

## Configuration

The web interface for configuration is available on port `8073`. Port `8055`
must also be open to allow websocket communication between the browser and
UniPager.

### Raspberry Pi
Make sure that the serial port is activated. To do this add `enable_uart=1` to
`/boot/config.txt`, remove `console=ttyAMA0,115200` from `/boot/cmdline.txt` and
reboot.

This is not needed for the RASPAGERV1 and Audio transmitter type.

## License

    UniPager
    Copyright (C) 2017  RWTH Amateurfunkgruppe

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
