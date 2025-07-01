#!/bin/bash

# Build script for @adventurelabs/scout-core

echo "Building @adventurelabs/scout-core..."

# Clean previous build
sudo yarn clean

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    sudo yarn install
fi

# Build the package
echo "Compiling TypeScript..."
sudo yarn build

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📦 Package is ready for publishing"
    echo ""
    echo "To publish:"
    echo "  sudo yarn publish"
    echo ""
    echo "To test locally:"
    echo "  sudo npm pack"
else
    echo "❌ Build failed!"
    exit 1
fi 