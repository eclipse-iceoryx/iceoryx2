#!/bin/bash
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

SCRIPT_START_TIME=$(date +%s)

# Parse Arguments
# ---------------
DRY_RUN=false
ARCH="x86_64"
IMAGE_DIR="$HOME/images/default"
SMP=2
MEMORY="1G"
BRIDGE="br0"
QEMU_BRIDGE_HELPER="/usr/lib/qemu/qemu-bridge-helper"
BRIDGE_CONF="/etc/qemu/bridge.conf"

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --arch)
            ARCH="$2"
            shift 2
            ;;
        --image-dir)
            IMAGE_DIR="$2"
            shift 2
            ;;
        --smp)
            SMP="$2"
            shift 2
            ;;
        --memory)
            MEMORY="$2"
            shift 2
            ;;
        --bridge)
            BRIDGE="$2"
            shift 2
            ;;
        --qemu-bridge-helper)
            QEMU_BRIDGE_HELPER="$2"
            shift 2
            ;;
        --bridge-conf)
            BRIDGE_CONF="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run                 Show what commands would be run without executing them"
            echo "  --arch ARCH               Set target architecture (default: x86_64)"
            echo "  --image-dir DIR           Set image directory (default: \$HOME/images/default)"
            echo "  --smp CORES               Set number of CPU cores (default: 2)"
            echo "  --memory SIZE             Set memory size (default: 1G)"
            echo "  --bridge BRIDGE           Set bridge name (default: br0)"
            echo "  --qemu-bridge-helper PATH Set qemu bridge helper path (default: /usr/lib/qemu/qemu-bridge-helper)"
            echo "  --bridge-conf PATH        Set bridge configuration path (default: /etc/qemu/bridge.conf)"
            echo "  --help, -h                Show this help message"
            echo ""
            echo "This script runs a QNX image in QEMU with bridge networking."
            echo "The QNX environment must be sourced before running this script."
            echo ""
            echo "Examples:"
            echo "  $0 # Run with defaults"
            echo "  $0 --arch aarch64 --smp 4 --memory 2G"
            echo "  $0 --image-dir /path/to/custom/images"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Configuration
# -------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Validate Dependencies and Environment
# ------------------------------------

if [ -z "$QNX_TARGET" ]; then
    echo -e "${RED}ERROR: QNX environment not detected${NC}"
    echo -e "${RED}Please source the QNX environment before running this script${NC}"
    echo -e "${YELLOW}Example: source \$HOME/qnx710/qnxsdp-env.sh${NC}"
    exit 1
fi

# Determine QNX version and toolchain path
QNX_VERSION=""
QNX_TOOLCHAIN=""
if [[ "$QNX_TARGET" == *"qnx710"* ]]; then
    QNX_VERSION="7.1"
    QNX_TOOLCHAIN=$(dirname "$(dirname "$QNX_TARGET")")
elif [[ "$QNX_TARGET" == *"qnx800"* ]]; then
    QNX_VERSION="8.0"
    QNX_TOOLCHAIN=$(dirname "$(dirname "$QNX_TARGET")")
else
    echo -e "${YELLOW}WARNING: Could not determine QNX version from QNX_TARGET${NC}"
    QNX_VERSION="Unknown"
    QNX_TOOLCHAIN=$(dirname "$(dirname "$QNX_TARGET")")
fi

# Set QEMU system based on architecture
case "$ARCH" in
    x86_64)
        QEMU_SYSTEM="qemu-system-x86_64"
        ;;
    aarch64)
        QEMU_SYSTEM="qemu-system-aarch64"
        ;;
    x86)
        QEMU_SYSTEM="qemu-system-i386"
        ;;
    *)
        QEMU_SYSTEM="qemu-system-$ARCH"
        ;;
esac

# Validate required files and tools
OUTPUT_DIR="$IMAGE_DIR/output"
SHARED_DIR="$IMAGE_DIR/shared"
DISK_IMAGE="$OUTPUT_DIR/disk-qemu.vmdk"
KERNEL_IMAGE="$OUTPUT_DIR/ifs.bin"
NET_SCRIPT="$QNX_TOOLCHAIN/host/common/mkqnximage/qemu/net.sh"

if ! command -v "$QEMU_SYSTEM" &> /dev/null; then
    echo -e "${RED}ERROR: $QEMU_SYSTEM not found in PATH${NC}"
    exit 1
fi

if [ ! -f "$DISK_IMAGE" ]; then
    echo -e "${RED}ERROR: Disk image not found: $DISK_IMAGE${NC}"
    echo -e "${YELLOW}Please create the QNX image first using create-qnx-image.sh${NC}"
    exit 1
fi

if [ ! -f "$KERNEL_IMAGE" ]; then
    echo -e "${RED}ERROR: Kernel image not found: $KERNEL_IMAGE${NC}"
    echo -e "${YELLOW}Please create the QNX image first using create-qnx-image.sh${NC}"
    exit 1
fi

if [ ! -f "$NET_SCRIPT" ]; then
    echo -e "${RED}ERROR: Network script not found: $NET_SCRIPT${NC}"
    exit 1
fi

if [ ! -f "$QEMU_BRIDGE_HELPER" ]; then
    echo -e "${YELLOW}WARNING: QEMU bridge helper not found: $QEMU_BRIDGE_HELPER${NC}"
    echo -e "${YELLOW}Network bridging may not work properly${NC}"
fi

# Generate random MAC address
MAC=$(printf "52:54:00:%02x:%02x:%02x" $(( $RANDOM & 0xff)) $(( $RANDOM & 0xff )) $(( $RANDOM & 0xff)))

# Display Configuration
# --------------------

echo -e "${BLUE}QNX Image Runner${NC}"
echo -e "${BLUE}================${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  QNX Version:      ${QNX_VERSION}"
echo -e "  Architecture:     ${ARCH}"
echo -e "  QEMU System:      ${QEMU_SYSTEM}"
echo -e "  Image Directory:  ${IMAGE_DIR}"
echo -e "  CPU Cores:        ${SMP}"
echo -e "  Memory:           ${MEMORY}"
echo -e "  Bridge:           ${BRIDGE}"
echo -e "  MAC Address:      ${MAC}"
echo ""
echo -e "${BLUE}Image Files:${NC}"
echo -e "  Disk Image:       ${DISK_IMAGE}"
echo -e "  Kernel Image:     ${KERNEL_IMAGE}"
echo -e "  Shared Directory: ${SHARED_DIR}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - QEMU will not be started${NC}"
    echo ""
fi

# Create Shared Directory
# ----------------------

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}Would create shared directory: ${SHARED_DIR}${NC}"
else
    echo -e "${BLUE}Creating shared directory...${NC}"
    mkdir -p "$SHARED_DIR"
    echo -e "${GREEN}✓ Shared directory created: $SHARED_DIR${NC}"
fi

echo ""

# Setup Network Bridge
# -------------------

NET_ARGS=""
if [ "$QNX_VERSION" = "8.0" ]; then
    NET_ARGS="bridge"
fi

NET_CMD="sudo $NET_SCRIPT $QEMU_BRIDGE_HELPER $BRIDGE_CONF $NET_ARGS"

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}Network setup command that would be executed:${NC}"
    echo -e "${YELLOW}$NET_CMD${NC}"
else
    echo -e "${BLUE}Setting up network bridge...${NC}"
    echo -e "${BLUE}Command: $NET_CMD${NC}"
    
    BRIDGE_START_TIME=$(date +%s)
    set +e
    eval "$NET_CMD"
    BRIDGE_EXIT_CODE=$?
    set -e
    BRIDGE_END_TIME=$(date +%s)
    BRIDGE_DURATION=$((BRIDGE_END_TIME - BRIDGE_START_TIME))
    
    if [ $BRIDGE_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ Network bridge setup completed (${BRIDGE_DURATION}s)${NC}"
    else
        echo -e "${YELLOW}⚠ Network bridge setup failed (${BRIDGE_DURATION}s, exit code: $BRIDGE_EXIT_CODE)${NC}"
        echo -e "${YELLOW}Continuing anyway - network may not work properly${NC}"
    fi
fi

echo ""

# Start QEMU
# ---------

QEMU_ARGS=(
    "-smp" "$SMP"
    "-m" "$MEMORY"
    "-drive" "file=$DISK_IMAGE,if=ide,id=drv0"
    "-hdb" "fat:rw:$SHARED_DIR"
    "-netdev" "bridge,br=$BRIDGE,id=net0"
    "-device" "e1000,netdev=net0,mac=$MAC"
    "-nographic"
    "-kernel" "$KERNEL_IMAGE"
    "-serial" "mon:stdio"
    "-object" "rng-random,filename=/dev/urandom,id=rng0"
    "-device" "virtio-rng-pci,rng=rng0"
)

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}QEMU command that would be executed:${NC}"
    echo -e "${YELLOW}$QEMU_SYSTEM ${QEMU_ARGS[*]}${NC}"
    echo ""
    echo -e "${YELLOW}Dry run completed. QEMU was not started.${NC}"
    exit 0
fi

echo -e "${BLUE}Starting QEMU...${NC}"
echo -e "${BLUE}Command: $QEMU_SYSTEM ${QEMU_ARGS[*]}${NC}"
echo ""
echo -e "${YELLOW}=== QNX Virtual Machine Starting ===${NC}"
echo -e "${YELLOW}Press Ctrl+A then X to exit QEMU${NC}"
echo -e "${YELLOW}Shared directory: $SHARED_DIR${NC}"
echo -e "${YELLOW}====================================${NC}"
echo ""

VM_START_TIME=$(date +%s)

# Execute QEMU
set +e
"$QEMU_SYSTEM" "${QEMU_ARGS[@]}"
QEMU_EXIT_CODE=$?
set -e

VM_END_TIME=$(date +%s)
VM_DURATION=$((VM_END_TIME - VM_START_TIME))

echo ""

if [ $QEMU_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ QEMU exited normally (${VM_DURATION}s)${NC}"
else
    echo -e "${YELLOW}⚠ QEMU exited with code $QEMU_EXIT_CODE (${VM_DURATION}s)${NC}"
fi

SCRIPT_END_TIME=$(date +%s)
TOTAL_DURATION=$((SCRIPT_END_TIME - SCRIPT_START_TIME))

# Print Summary
# -------------

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}SUMMARY${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "QNX Version: $QNX_VERSION"
echo -e "Architecture: $ARCH"
echo -e "VM Runtime: ${VM_DURATION}s"
echo -e "Total execution time: ${TOTAL_DURATION}s"
echo -e "${GREEN}✓ Session completed${NC}"

