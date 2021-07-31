#!/usr/bin/env bash

export DEBIAN_FRONTEND=noninteractive

echo "wireshark-common wireshark-common/install-setuid boolean false" | sudo debconf-set-selections

apt-get update
apt-get upgrade -y
apt-get install -y build-essential clang llvm libclang-dev linux-tools-oem \
  linux-tools-5.4.0-58-generic chrony wireshark xauth

su vagrant << EOF
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # Installing cargo-with allows for `cargo with "strace -fe bpf" -- test` while testing
  source /home/vagrant/.cargo/env
  cargo install cargo-with
EOF

echo "192.168.5.4	tb-client" >> /etc/hosts
echo "192.168.5.3	tb-dmz" >> /etc/hosts
echo "192.168.5.2	tb-server" >> /etc/hosts

echo "allow 192.168.5.0/24" >> /etc/chrony/chrony.conf
echo 'alias scargo="sudo -E $HOME/.cargo/bin/cargo"' >> ~/.bashrc

systemctl restart chrony.service

