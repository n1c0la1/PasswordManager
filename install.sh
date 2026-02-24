#!/bin/bash

# Check if pre-built binary exists (for USB/offline install)
if [ -f "./password_manager" ]; then
    echo "Found pre-built binary 'password_manager'. Skipping build."
    BINARY_SOURCE="./password_manager"
elif [ -f "./pw" ]; then
    echo "Found pre-built binary 'pw'. Skipping build."
    BINARY_SOURCE="./pw"
else
    # Build the project
    if command -v cargo &> /dev/null; then
        echo "Building password_manager..."
        cargo build --release
        if [ $? -ne 0 ]; then
            echo "Build failed. Please check your Rust installation."
            exit 1
        fi
        BINARY_SOURCE="target/release/password_manager"
    else
        echo "Error: 'cargo' not found and no pre-built binary found."
        echo "For offline installation, please place the 'password_manager' binary in this folder."
        exit 1
    fi
fi

# Determine install location -> Standard local binary locations
# Binary then is owned by root, but can be executed by anyone.
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR" 2>/dev/null
    if [ $? -ne 0 ]; then
        echo "Error: Cannot create $INSTALL_DIR. You need sudo permissions."
        echo "Please run this script with sudo:"
        echo ""
        echo "    sudo bash install.sh"
        echo ""
        exit 1
    fi
fi

# Check if we have write permissions to INSTALL_DIR
if [ ! -w "$INSTALL_DIR" ]; then
    echo "Error: You don't have write permissions to $INSTALL_DIR"
    echo "Please run this script with sudo:"
    echo ""
    echo "    sudo bash install.sh"
    echo ""
    exit 1
fi

# Install
echo "Installing to $INSTALL_DIR..."
cp "$BINARY_SOURCE" "$INSTALL_DIR/pw"
chmod +x "$INSTALL_DIR/pw" # Should normally be set during `cargo build --release`, just as backup.

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
            echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
            echo ""
            echo "Then run: source ~/.zshrc (or your shell config)"
            ;;
    esac
else
    echo "Installation failed. You might need sudo permissions if installing to /usr/local/bin."
    echo "Try: sudo cp target/release/password_manager /usr/local/bin/pw"
fi
