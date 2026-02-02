#!/bin/bash

# Build the project
echo "Building password_manager..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed. Please check your Rust installation."
    exit 1
fi

# Determine install location - prefer local bin to avoid sudo
INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
fi

# Install
echo "Installing to $INSTALL_DIR..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cp "$SCRIPT_DIR/target/release/password_manager" "$INSTALL_DIR/pw"

if [ $? -eq 0 ]; then
    echo "Installation successful! You can now use 'pw' from the terminal."
    
    # Check if INSTALL_DIR is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo ""
            echo "WARNING: $INSTALL_DIR is NOT in your PATH."
            echo "To use 'pw', add this to your shell configuration (e.g. ~/.zshrc or ~/.bashrc):"
            echo ""
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
            echo "Then run: source ~/.zshrc (or your shell config)"
            ;;
    esac
else
    echo "Installation failed. You might need sudo permissions if installing to /usr/local/bin."
    echo "Try: sudo cp target/release/password_manager /usr/local/bin/pw"
fi
