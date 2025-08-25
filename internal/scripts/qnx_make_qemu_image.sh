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
VM_HOSTNAME="$ARCH-qnx-vm"
VM_IPV4_ADDR="172.31.1.11"
IMAGE_DIR="$HOME/images/default"
SYS_SIZE=256
SYS_INODES=24000
DATA_SIZE=256
DATA_INODES=24000

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
        --hostname)
            VM_HOSTNAME="$2"
            shift 2
            ;;
        --ip)
            VM_IPV4_ADDR="$2"
            shift 2
            ;;
        --image-dir)
            IMAGE_DIR="$2"
            shift 2
            ;;
        --sys-size)
            SYS_SIZE="$2"
            shift 2
            ;;
        --sys-inodes)
            SYS_INODES="$2"
            shift 2
            ;;
        --data-size)
            DATA_SIZE="$2"
            shift 2
            ;;
        --data-inodes)
            DATA_INODES="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run           Show what commands would be run without executing them"
            echo "  --arch ARCH         Set target architecture (default: x86_64)"
            echo "  --hostname HOSTNAME Set VM hostname (default: x86_64-qnx-vm)"
            echo "  --ip IP_ADDRESS     Set VM IP address (default: 172.31.1.11)"
            echo "  --image-dir DIR     Set output directory for images (default: \$HOME/images/default)"
            echo "  --sys-size SIZE     Set system partition size in MB (default: 256)"
            echo "  --sys-inodes COUNT  Set system partition inode count (default: 24000)"
            echo "  --data-size SIZE    Set data partition size in MB (default: 256)"
            echo "  --data-inodes COUNT Set data partition inode count (default: 24000)"
            echo "  --help, -h          Show this help message"
            echo ""
            echo "This script creates a QNX image for QEMU using mkqnximage."
            echo "The QNX environment must be sourced before running this script."
            echo ""
            echo "Examples:"
            echo "  $0 # Create image with defaults"
            echo "  $0 --arch aarch64 --hostname arm-qnx-vm"
            echo "  $0 --hostname my-qnx-vm --ip 192.168.1.100"
            echo "  $0 --sys-size 512 --data-size 1024   # Larger partitions"
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

if ! command -v mkqnximage &> /dev/null; then
    echo -e "${RED}ERROR: mkqnximage not found in PATH${NC}"
    echo -e "${RED}Ensure the QNX environment is properly sourced${NC}"
    exit 1
fi

# Determine QNX version
QNX_VERSION=""
if [[ "$QNX_TARGET" == *"qnx710"* ]]; then
    QNX_VERSION="7.1"
elif [[ "$QNX_TARGET" == *"qnx800"* ]]; then
    QNX_VERSION="8.0"
else
    echo -e "${YELLOW}WARNING: Could not determine QNX version from QNX_TARGET${NC}"
    QNX_VERSION="Unknown"
fi

# Display Configuration
# --------------------

echo -e "${BLUE}QNX Image Creation Script${NC}"
echo -e "${BLUE}=========================${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  QNX Version:     ${QNX_VERSION}"
echo -e "  Architecture:    ${ARCH}"
echo -e "  Hostname:        ${VM_HOSTNAME}"
echo -e "  IP Address:      ${VM_IPV4_ADDR}"
echo -e "  Output Directory: ${IMAGE_DIR}"
echo -e "  System Size:     ${SYS_SIZE} MB (${SYS_INODES} inodes)"
echo -e "  Data Size:       ${DATA_SIZE} MB (${DATA_INODES} inodes)"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No image will be created${NC}"
    echo ""
fi

# Create Image Directory
# ---------------------

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}Would create directory: ${IMAGE_DIR}${NC}"
else
    echo -e "${BLUE}Creating output directory...${NC}"
    mkdir -p "$IMAGE_DIR"
    cd "$IMAGE_DIR"
    echo -e "${GREEN}✓ Directory created: $(pwd)${NC}"
fi

echo ""

# Build mkqnximage command
MKQNX_CMD="mkqnximage"
MKQNX_ARGS=(
    "--noprompt"
    "--hostname=$VM_HOSTNAME"
    "--type=qemu"
    "--arch=$ARCH"
    "--ip=$VM_IPV4_ADDR"
    "--sshd-pregen=yes"
    "--sys-size=$SYS_SIZE"
    "--sys-inodes=$SYS_INODES"
    "--data-size=$DATA_SIZE"
    "--data-inodes=$DATA_INODES"
)

if [[ "$QNX_VERSION" == "7.1" ]]; then
    MKQNX_ARGS+=("--telnet=yes")
fi

# Create QNX Image
# ---------------

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}Command that would be executed:${NC}"
    echo -e "${YELLOW}cd $IMAGE_DIR${NC}"
    echo -e "${YELLOW}$MKQNX_CMD ${MKQNX_ARGS[*]}${NC}"
    echo ""
    echo -e "${YELLOW}Dry run completed. No image was created.${NC}"
    exit 0
fi

echo -e "${BLUE}Creating QNX image...${NC}"
echo -e "${BLUE}Command: $MKQNX_CMD ${MKQNX_ARGS[*]}${NC}"
echo ""

IMAGE_START_TIME=$(date +%s)

set +e  # Don't exit on error
"$MKQNX_CMD" "${MKQNX_ARGS[@]}"
EXIT_CODE=$?
set -e

IMAGE_END_TIME=$(date +%s)
IMAGE_DURATION=$((IMAGE_END_TIME - IMAGE_START_TIME))

echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ Image creation completed successfully (${IMAGE_DURATION}s)${NC}"
else
    echo -e "${RED}✗ Image creation failed (${IMAGE_DURATION}s, exit code: $EXIT_CODE)${NC}"
    exit $EXIT_CODE
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
echo -e "Output Directory: $IMAGE_DIR"
echo -e "Total execution time: ${TOTAL_DURATION}s"
echo -e "${GREEN}🎉 QNX image created successfully!${NC}"

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Use the created image files with QEMU"
echo -e "  2. VM will be accessible at IP: $VM_IPV4_ADDR"
echo -e "  3. SSH and telnet are pre-configured"
