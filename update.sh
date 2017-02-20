#!/bin/bash
echo "Updating Compiler..."
rustup update
echo "Updating Rustpager sources..."
git pull
echo "Compiling new Version of Rustpager...."
cargo build --release
echo "Installing new Version of Rustpager..."
echo "Stopping running instance of Rustpager..."
sudo systemctl stop rustpager
sudo ./install.sh
echo "Install completed. If there were no error messages, you can start the new version by typing"
echo "sudo systemctl start rustpager"
echo "Done."
