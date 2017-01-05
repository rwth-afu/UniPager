# RustPager

Universal POCSAG transmitter controller written in Rust.

## Web Interface
The web interface is available on port `8073`. Port `8055` must also be
open to allow websocket communication between the browser and RustPager.

## Compiling
Install rust:

```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

Clone the source:

```bash
git clone https://github.com/7h0ma5/RustPager.git
```

Start the build:

```bash
cd RustPager
cargo build --release
```

Run:

```bash
./target/release/rustpager
```

## Cross Compiling

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
git clone https://github.com/7h0ma5/RustPager.git
```

Start the build:

```bash
cd RustPager
cargo build --target $TARGET --release 
```

The cross-compiled binary will be created at `./target/$TARGET/release/rustpager`.

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
