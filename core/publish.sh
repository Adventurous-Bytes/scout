#!/bin/bash

# Publish script for @adventurelabs/scout-core

set -e  # Exit on any error

echo "🚀 Publishing @adventurelabs/scout-core..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    print_error "package.json not found. Please run this script from the scout/core directory."
    exit 1
fi

# Get current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo "Current version: $CURRENT_VERSION"

# Increment patch version
print_status "Incrementing patch version..."
NEW_VERSION=$(npm version patch --no-git-tag-version)
print_status "New version: $NEW_VERSION"

# Check if dist directory exists
if [ ! -d "dist" ]; then
    print_warning "dist directory not found. Building package..."
    sudo yarn build
fi

# Check if user is logged in to npm
print_status "Checking npm login status..."
if ! npm whoami > /dev/null 2>&1; then
    print_warning "Not logged in to npm. Please log in:"
    echo ""
    echo "You'll need to provide:"
    echo "  - Username"
    echo "  - Password (or access token)"
    echo "  - Email"
    echo ""
    echo "If you don't have an npm account, create one at: https://www.npmjs.com/signup"
    echo ""
    read -p "Press Enter to continue with npm login..."
    npm login
else
    print_status "Already logged in to npm as: $(npm whoami)"
fi

# Show package info before publishing
echo ""
print_status "Package information:"
echo "  Name: $(node -p "require('./package.json').name")"
echo "  Version: $NEW_VERSION"
echo "  Description: $(node -p "require('./package.json').description")"
echo ""

# Confirm before publishing
read -p "Do you want to publish this package to npm? (y/N): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Publishing cancelled."
    print_warning "Reverting version change..."
    npm version $CURRENT_VERSION --no-git-tag-version
    exit 0
fi

# Publish the package
print_status "Publishing to npm..."
if sudo npm publish --access public; then
    print_status "Successfully published @adventurelabs/scout-core!"
    echo ""
    echo "🎉 Your package is now available at:"
    echo "   https://www.npmjs.com/package/@adventurelabs/scout-core"
    echo ""
    echo "To install in other projects:"
    echo "   yarn add @adventurelabs/scout-core"
    echo "   # or"
    echo "   npm install @adventurelabs/scout-core"
else
    print_error "Failed to publish package."
    print_warning "Reverting version change..."
    npm version $CURRENT_VERSION --no-git-tag-version
    exit 1
fi 