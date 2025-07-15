#!/bin/bash

# Build script for iceoryx2 Flutter example
# This script builds the iceoryx2 C FFI library and prepares the Flutter environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building iceoryx2 Flutter Example${NC}"
echo "=================================="

# Check if we're in the right directory
if [ ! -f "../../../Cargo.toml" ]; then
    echo -e "${RED}Error: This script must be run from examples/flutter/publish_subscribe directory${NC}"
    exit 1
fi

# Step 1: Build iceoryx2 C FFI library
echo -e "${YELLOW}Step 1: Building iceoryx2 C FFI library...${NC}"
cd ../../../
cargo build --release -p iceoryx2-ffi
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ iceoryx2 C FFI library built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build iceoryx2 C FFI library${NC}"
    exit 1
fi

# Check if the library was created
if [ -f "target/release/libiceoryx2_ffi.so" ]; then
    echo -e "${GREEN}✓ Found libiceoryx2_ffi.so${NC}"
else
    echo -e "${RED}✗ libiceoryx2_ffi.so not found${NC}"
    exit 1
fi

# Check if headers were generated
if [ -f "target/release/iceoryx2-ffi-cbindgen/include/iox2/iceoryx2.h" ]; then
    echo -e "${GREEN}✓ Found C headers${NC}"
else
    echo -e "${RED}✗ C headers not found${NC}"
    exit 1
fi

# Step 2: Setup Flutter environment
echo -e "${YELLOW}Step 2: Setting up Flutter environment...${NC}"
cd examples/flutter/publish_subscribe

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    echo -e "${RED}✗ Flutter is not installed or not in PATH${NC}"
    echo "Please install Flutter: https://flutter.dev/docs/get-started/install"
    exit 1
fi

echo -e "${GREEN}✓ Flutter found${NC}"

# Get Flutter dependencies
echo -e "${YELLOW}Getting Flutter dependencies...${NC}"
flutter pub get
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Flutter dependencies installed${NC}"
else
    echo -e "${RED}✗ Failed to install Flutter dependencies${NC}"
    exit 1
fi

# Check if Linux desktop is enabled
flutter config --enable-linux-desktop > /dev/null 2>&1

echo ""
echo -e "${GREEN}Build completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Usage:${NC}"
echo "To run the combined app:"
echo "  flutter run -d linux"
echo ""
echo "To run publisher only:"
echo "  flutter run -d linux lib/publisher.dart"
echo ""
echo "To run subscriber only:"
echo "  flutter run -d linux lib/subscriber.dart"
echo ""
echo -e "${YELLOW}Note:${NC} Make sure to start the subscriber before the publisher for best results."
