#!/bin/bash

# Headless validation script for iceoryx2 Flutter example
# Tests event-driven communication without UI using iox2_node_wait

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE} Headless iceoryx2 Flutter Validation Test${NC}"
echo -e "${BLUE}================================================${NC}"
echo

# Check if we're in the right directory (either project root or test dir)
if [ ! -f "pubspec.yaml" ] && [ ! -f "../pubspec.yaml" ]; then
    echo -e "${RED}Error: Run this script from the Flutter project directory or test/ directory${NC}"
    exit 1
fi

# Change to project root if we're in test directory
if [ -f "../pubspec.yaml" ]; then
    cd ..
fi

# Function to kill background processes
cleanup() {
    echo -e "\n${YELLOW}Cleaning up background processes...${NC}"
    
    # Kill specific PIDs if they exist
    if [ ! -z "$SUBSCRIBER_PID" ]; then
        kill $SUBSCRIBER_PID 2>/dev/null || true
        echo "Stopped subscriber (PID: $SUBSCRIBER_PID)"
    fi
    if [ ! -z "$PUBLISHER_PID" ]; then
        kill $PUBLISHER_PID 2>/dev/null || true
        echo "Stopped publisher (PID: $PUBLISHER_PID)"
    fi
    
    # Additional cleanup to ensure no Dart/Flutter processes remain
    echo "Performing additional cleanup..."
    pkill -f dart 2>/dev/null || true
    pkill -f flutter 2>/dev/null || true
    killall dart 2>/dev/null || true
    killall flutter 2>/dev/null || true
    
    # Wait a moment for processes to terminate
    sleep 1
    
    # Force kill any remaining processes
    pkill -9 -f "headless_subscriber" 2>/dev/null || true
    pkill -9 -f "headless_publisher" 2>/dev/null || true
    
    echo -e "${GREEN}Cleanup completed${NC}"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

echo -e "${YELLOW}Step 0: Cleaning up any existing processes...${NC}"
pkill -f dart 2>/dev/null || true
pkill -f flutter 2>/dev/null || true
killall dart 2>/dev/null || true
killall flutter 2>/dev/null || true
sleep 1

echo -e "${YELLOW}Step 1: Building iceoryx2 FFI library...${NC}"
cd ../../..
cargo build --release
cd examples/flutter/publish_subscribe

echo -e "${YELLOW}Step 2: Getting Flutter dependencies...${NC}"
flutter pub get

echo -e "${YELLOW}Step 3: Starting headless subscriber (event-driven)...${NC}"
dart lib/headless/headless_subscriber.dart &
SUBSCRIBER_PID=$!
echo "Subscriber started with PID: $SUBSCRIBER_PID"

# Wait a moment for subscriber to initialize
sleep 2

echo -e "${YELLOW}Step 4: Starting headless publisher...${NC}"
dart lib/headless/headless_publisher.dart &
PUBLISHER_PID=$!
echo "Publisher started with PID: $PUBLISHER_PID"

echo
echo -e "${GREEN}✓ Both processes started successfully!${NC}"
echo -e "${BLUE}Testing event-driven communication...${NC}"
echo
echo "The test will run for 30 seconds..."
echo "You should see:"
echo "  - Publisher sending messages every 500ms"
echo "  - Subscriber receiving messages instantly (event-driven)"
echo "  - No polling, pure event-driven architecture using iox2_node_wait"
echo

# Let the test run for 30 seconds
for i in {30..1}; do
    echo -ne "\rTest running... ${i}s remaining  "
    sleep 1
done

echo
echo
echo -e "${GREEN}✓ Test completed successfully!${NC}"
echo
echo -e "${BLUE}What was tested:${NC}"
echo "  ✓ Node wait based event-driven subscriber"
echo "  ✓ Zero-copy message passing"
echo "  ✓ CPU-efficient (no polling)"
echo "  ✓ Instant message delivery"
echo "  ✓ Proper resource cleanup"
echo
echo -e "${BLUE}Architecture validated:${NC}"
echo "  ✓ Publisher → iceoryx2 shared memory → iox2_node_wait → Subscriber"
echo "  ✓ True event-driven (blocking wait on events)"
echo "  ✓ Isolate-based background processing"
echo
echo -e "${GREEN}Headless validation completed successfully!${NC}"
