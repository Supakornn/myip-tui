#!/bin/bash

set -e 

echo "Setting up MyIP network interface viewer..."

if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed."
fi

echo "Building the project..."
cargo build --release

echo "Making the binary accessible..."
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "Adding $BIN_DIR to PATH in your profile..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.profile"
    echo "Please run 'source $HOME/.profile' or restart your terminal to update your PATH."
fi

cp target/release/myip "$BIN_DIR/"
chmod +x "$BIN_DIR/myip"

echo "Installation complete! You can now run 'myip' to view your network interfaces." 