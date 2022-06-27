# arm-unknown-linux-gnueabihf but with GLIBC=2.27
# Made by 1. cloning the cross-rs repo, 2. Running crosstools-ng/configure.sh with `export GLIBC_VERSION="2.27.0"` 3. Building arm-unknown-linux-gnueabihf dockerfile and tagging it "cross-custom/armhf:latest"
FROM cross-custom/armhf:latest

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y libudev-dev:armhf && \
    cp /usr/include/libudev.h /x-tools/arm-unknown-linux-gnueabihf/arm-unknown-linux-gnueabihf/sysroot/usr/include/libudev.h

ENV LD_LIBRARY_PATH=/usr/arm-linux-gnueabihf/arm-linux-gnueabihf/libc/lib:/usr/arm-linux-gnueabihf/arm-linux-gnueabihf/lib:/usr/arm-linux-gnueabihf/lib:/lib/arm-linux-gnueabihf \
    PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig/:${PKG_CONFIG_PATH}"
