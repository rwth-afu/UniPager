# RustPager

[![Build Status](https://img.shields.io/travis/rwth-afu/RustPager.svg?style=flat)](https://travis-ci.org/rwth-afu/RustPager)
[![GitHub issues](https://img.shields.io/github/issues/rwth-afu/RustPager.svg?style=flat)](https://github.com/rwth-afu/RustPager/issues)
[![GitHub release](https://img.shields.io/github/release/rwth-afu/RustPager.svg?style=flat)](https://github.com/rwth-afu/RustPager/releases)

Universal POCSAG transmitter controller written in Rust.

## Compilation
Be aware: Install with Raspbian wheezy will fail, you need jessie. Using a fresh installation of Your Operating System will minimize the chance of running into truble.

It is recommended to update your OS before installing:

```bash
sudo apt-get update
sudo apt-get upgrade
```

Install rust:

```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

Now reboot OR log off to make the Rust-Toolchain available:

```bash
sudo reboot
OR
logout
(SSH sessions will be closed)
```

Log in again and clone the source:

```bash
git clone https://github.com/rwth-afu/RustPager.git
```

If this command fails, You may need to install git and try again:

```bash
sudo apt-get install git
```

Start the build:

```bash
cd RustPager
cargo build --release
```

Run the install script:

```bash
sudo ./install.sh
```

Autostart for RustPager:

```bash
sudo systemctl enable rustpager
```

Finally do a reboot to test the Autostart sequence of RustPager

```bash
sudo reboot
```

Be aware: Must be run with root privileges. Also directory /etc/rustpager must exist and be writeable by root.

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
git clone https://github.com/rwth-afu/RustPager.git
```

Start the build:

```bash
cd RustPager
cargo build --target $TARGET --release 
```

The cross-compiled binary will be created at `./target/$TARGET/release/rustpager`.

## Configuration
The web interface for configuration is available on port `8073`. Port `8055`
must also be open to allow websocket communication between the browser and
RustPager.

### Raspberry Pi
Make sure that the serial port is activated. To do this add `enable_uart=1` to
`/boot/config.txt`, remove `console=ttyAMA0,115200` from `/boot/cmdline.txt` and
reboot.

This is not needed for the RASPAGERV1 and Audio transmitter type.

## License

    RustPager
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
