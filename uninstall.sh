#!/bin/bash

# Define binary name
BINARY_NAME="pw"

echo "Uninstalling $BINARY_NAME..."

# Remove from user local bin
if [ -f "$HOME/.local/bin/$BINARY_NAME" ]; then
    rm "$HOME/.local/bin/$BINARY_NAME"
    echo "Removed $HOME/.local/bin/$BINARY_NAME"
else
    echo "No binary found in $HOME/.local/bin"
fi

# Remove from system bin (requires sudo if present)
if [ -f "/usr/local/bin/$BINARY_NAME" ]; then
    echo "Need sudo to remove /usr/local/bin/$BINARY_NAME"
    sudo rm "/usr/local/bin/$BINARY_NAME"
    echo "Removed /usr/local/bin/$BINARY_NAME"
fi

if [ -f "/usr/bin/$BINARY_NAME" ]; then
    echo "Need sudo to remove /usr/bin/$BINARY_NAME"
    sudo rm "/usr/bin/$BINARY_NAME"
    echo "Removed /usr/bin/$BINARY_NAME"
fi

echo ""
echo "Do you want to delete all your vaults and data?"
read -p "Type 'DELETE' to confirm: " confirmation
echo ""

if [[ "$confirmation" == "DELETE" ]]; then
    # Delete for Mac
    if [ -d "$HOME/Library/Application Support/password_manager" ]; then
        rm -rf "$HOME/Library/Application Support/password_manager"
        echo "Removed data directory (macOS)"
    fi
    
    # Delete for Linux
    if [ -d "$HOME/.local/share/password_manager" ]; then
        rm -rf "$HOME/.local/share/password_manager"
        echo "Removed data directory (Linux)"
    fi
    
    # Delete local development vaults if any
    if [ -d "vaults" ]; then
        read -p "Also remove local 'vaults/' directory in current folder? (y/N) " local_confirm
        if [[ $local_confirm =~ ^[Yy]$ ]]; then
            rm -rf "vaults"
            echo "Removed local vaults directory"
        fi
    fi

    echo "All data removed."
else
    echo "Data preserved."
fi

echo "Uninstallation complete."
