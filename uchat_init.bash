#!/bin/bash

# Update system dependencies
echo "Updating system..."
sudo apt update && sudo apt upgrade -y

# Install curl
echo "Installing curl..."
sudo apt install curl -y

# Install Rust
echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env  # Add Rust to the PATH

# Verify Rust installation
echo "Verifying Rust installation..."
rustup --version

# Install pkg-config
echo "Installing pkg-config..."
sudo apt install pkg-config -y

# Install wasm-bindgen
echo "Installing wasm-bindgen..."
cargo install wasm-bindgen-cli

# Install PostgreSQL and contrib package
echo "Installing PostgreSQL..."
sudo apt install postgresql postgresql-contrib -y

# Install libssl-dev for SSL support
echo "Installing libssl-dev..."
sudo apt install libssl-dev -y

# Start PostgreSQL service
echo "Starting PostgreSQL service..."
sudo systemctl start postgresql.service

# Install libpq-dev for PostgreSQL
echo "Installing libpq-dev..."
sudo apt-get install libpq-dev -y

# Install npm globally
echo "Installing npm..."
sudo apt install npm -y

# Install Node.js
echo "Installing Node.js..."
sudo apt install nodejs -y

# Install NVM Tooling
echo "Installing NVM..."
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash
source ~/.bashrc  # Reload shell configuration

# Install specific LTS version of Node.js with nvm
echo "Installing Node.js LTS version using NVM..."
nvm install 21.7.3

# Install build-essential for C compiler and build tools
echo "Installing build-essential..."
sudo apt install build-essential -y

echo "All dependencies have been installed successfully!"
