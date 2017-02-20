#!/bin/bash
echo "Hint: Must be run as root! Try sudo ./install.sh if not working."
echo "Installing RustPager..."
cp rustpager.service /etc/systemd/system/rustpager.service
chown root /etc/systemd/system/rustpager.service
chmod 644 /etc/systemd/system/rustpager.service
cp target/release/rustpager /usr/local/bin/rustpager
chown root /usr/local/bin/rustpager
chmod 744 /usr/local/bin/rustpager
if [ ! -d "/etc/rustpager" ]; then
  mkdir /etc/rustpager
else
  echo "Directory /etc/rustpager already present."
fi
sudo systemctl daemon-reload
echo "Installation complete. Use --> sudo systemctl start rustpager <-- to start the service."
echo "Use --> sudo systemctl enable rustpager <-- to enable start at boot time."
echo "Config-Interface is avaiable at http://URLOfThisHost:8073"
echo "Happy paging..."
