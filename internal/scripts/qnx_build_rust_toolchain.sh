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
RUST_VERSION="1.90.0"
RUSTDIR="$HOME/source/rust"
IMAGE_DIR="$HOME/images/default"
BUILD_REMOTE_TOOLS=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --rust-version)
            RUST_VERSION="$2"
            shift 2
            ;;
        --rustdir)
            RUSTDIR="$2"
            shift 2
            ;;
        --image-dir)
            IMAGE_DIR="$2"
            shift 2
            ;;
        --toolchain-name)
            TOOLCHAIN_NAME="$2"
            shift 2
            ;;
        --no-remote-tools)
            BUILD_REMOTE_TOOLS=false
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run              Show what commands would be run without executing them"
            echo "  --rust-version VERSION Set Rust version to build (default: 1.88.0)"
            echo "  --rustdir DIR          Set Rust source directory (default: \$HOME/source/rust)"
            echo "  --image-dir DIR        Set image directory for remote tools (default: \$HOME/images/default)"
            echo "  --toolchain-name NAME  Set rustup toolchain name (default: qnx-custom)"
            echo "  --no-remote-tools      Skip building remote testing tools"
            echo "  --help, -h             Show this help message"
            echo ""
            echo "This script builds a custom Rust compiler for QNX targets."
            echo "The QNX environment must be sourced before running this script."
            echo ""
            echo "Examples:"
            echo "  $0 # Build with defaults"
            echo "  $0 --rust-version 1.90.0 --no-remote-tools"
            echo "  $0 --rustdir /custom/rust/path --toolchain-name my-qnx"
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

# Determine QNX version and set up targets
QNX_VERSION=""
QNX_TOOLCHAIN=""
if [[ "$QNX_TARGET" == *"qnx710"* ]]; then
    QNX_VERSION="7.1"
    QNX_TOOLCHAIN=$(dirname "$(dirname "$QNX_TARGET")")
    TARGETS="aarch64-unknown-nto-qnx710,x86_64-pc-nto-qnx710,x86_64-unknown-linux-gnu"
elif [[ "$QNX_TARGET" == *"qnx800"* ]]; then
    QNX_VERSION="8.0"
    QNX_TOOLCHAIN=$(dirname "$(dirname "$QNX_TARGET")")
    TARGETS="aarch64-unknown-nto-qnx800,x86_64-pc-nto-qnx800"
else
    echo -e "${RED}ERROR: Could not determine QNX version from QNX_TARGET${NC}"
    exit 1
fi

# Validate dependencies
if ! command -v git &> /dev/null; then
    echo -e "${RED}ERROR: git is required but not installed${NC}"
    exit 1
fi

if ! command -v rustup &> /dev/null; then
    echo -e "${RED}ERROR: rustup is required but not installed${NC}"
    exit 1
fi

if ! command -v qcc &> /dev/null; then
    echo -e "${RED}ERROR: qcc not found - ensure QNX environment is properly sourced${NC}"
    exit 1
fi

SHARED_DIR="$IMAGE_DIR/shared"
TOOLCHAIN_BIN_DIR="$QNX_TOOLCHAIN/host/linux/x86_64/usr/bin"

if [ -z "$TOOLCHAIN_NAME" ]; then
    if [ "$QNX_VERSION" = "7.1" ]; then
        TOOLCHAIN_NAME="qnx710"
    elif [ "$QNX_VERSION" = "8.0" ]; then
        TOOLCHAIN_NAME="qnx800"
    else
        TOOLCHAIN_NAME="qnx"
    fi
fi

# Display Configuration
# --------------------

echo -e "${BLUE}Rust Compiler Builder for QNX${NC}"
echo -e "${BLUE}==============================${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  QNX Version:       ${QNX_VERSION}"
echo -e "  QNX Toolchain:     ${QNX_TOOLCHAIN}"
echo -e "  Rust Version:      ${RUST_VERSION}"
echo -e "  Rust Directory:    ${RUSTDIR}"
echo -e "  Build Targets:     ${TARGETS}"
echo -e "  Toolchain Name:    ${TOOLCHAIN_NAME}"
echo -e "  Build Remote Tools: ${BUILD_REMOTE_TOOLS}"
if [ "$BUILD_REMOTE_TOOLS" = true ]; then
    echo -e "  Image Directory:   ${IMAGE_DIR}"
    echo -e "  Shared Directory:  ${SHARED_DIR}"
    echo -e "  Toolchain Bin:     ${TOOLCHAIN_BIN_DIR}"
fi
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No build will be performed${NC}"
    echo ""
fi

# Clone Rust Source
# ----------------

if [ "$DRY_RUN" = true ]; then
    echo -e "${BLUE}Commands that would be executed:${NC}"
    if [ -d "$RUSTDIR" ]; then
        echo -e "${YELLOW}cd $RUSTDIR && git checkout $RUST_VERSION${NC}"
    else
        echo -e "${YELLOW}git clone https://github.com/rust-lang/rust.git -b $RUST_VERSION --depth 1 $RUSTDIR${NC}"
    fi
else
    if [ -d "$RUSTDIR" ]; then
        echo -e "${BLUE}Updating existing Rust source...${NC}"
        cd "$RUSTDIR"
        git fetch origin "$RUST_VERSION"
        git checkout "$RUST_VERSION"
        echo -e "${GREEN}✓ Rust source updated to version $RUST_VERSION${NC}"
    else
        echo -e "${BLUE}Cloning Rust source...${NC}"
        CLONE_START_TIME=$(date +%s)
        git clone https://github.com/rust-lang/rust.git -b "$RUST_VERSION" --depth 1 "$RUSTDIR"
        CLONE_END_TIME=$(date +%s)
        CLONE_DURATION=$((CLONE_END_TIME - CLONE_START_TIME))
        echo -e "${GREEN}✓ Rust source cloned (${CLONE_DURATION}s)${NC}"
        cd "$RUSTDIR"
    fi
fi

echo ""

# Configure Build
# --------------

CONFIG_FILE="$RUSTDIR/config.toml"

echo -e "${BLUE}Configuring build...${NC}"

if [ "$QNX_VERSION" = "8.0" ]; then
    cat > "$CONFIG_FILE" << 'EOF'
[build]
extended = true
EOF
else
    echo -e "[build]\nextended = true" > "$CONFIG_FILE"
fi
    echo -e "${GREEN}✓ Build configuration written to $CONFIG_FILE${NC}"

echo ""

# Build Rust Compiler
# -------------------

if [ "$DRY_RUN" = true ]; then
echo -e "${BLUE}Compiler build command that would be executed:${NC}"

   echo -e "${YELLOW}cd $RUSTDIR${NC}"

   echo -e " "
   
   echo -e "${YELLOW}# Define build environment${NC}"
   echo -e "${YELLOW}export build_env='${NC}"
   if [ "$QNX_VERSION" = "7.1" ]; then
        echo -e "${YELLOW}    CC_x86_64_pc_nto_qnx710=qcc${NC}"
        echo -e "${YELLOW}    CFLAGS_x86_64_pc_nto_qnx710=-Vgcc_ntox86_64_cxx${NC}"
        echo -e "${YELLOW}    CXX_x86_64_pc_nto_qnx710=qcc${NC}"
        echo -e "${YELLOW}    AR_x86_64_pc_nto_qnx710=ntox86_64-ar${NC}"
        echo -e "${YELLOW}    CC_aarch64_unknown_nto_qnx710=qcc${NC}"
        echo -e "${YELLOW}    CFLAGS_aarch64_unknown_nto_qnx710=-Vgcc_ntoaarch64le_cxx${NC}"
        echo -e "${YELLOW}    CXX_aarch64_unknown_nto_qnx710=qcc${NC}"
        echo -e "${YELLOW}    AR_aarch64_unknown_nto_qnx710=ntoaarch64-ar${NC}"
    elif [ "$QNX_VERSION" = "8.0" ]; then
        echo -e "${YELLOW}    CC_x86_64_pc_nto_qnx800=qcc${NC}"
        echo -e "${YELLOW}    CFLAGS_x86_64_pc_nto_qnx800=-Vgcc_ntox86_64_cxx${NC}"
        echo -e "${YELLOW}    CXX_x86_64_pc_nto_qnx800=qcc${NC}"
        echo -e "${YELLOW}    AR_x86_64_pc_nto_qnx800=ntox86_64-ar${NC}"
        echo -e "${YELLOW}    CC_aarch64_unknown_nto_qnx800=qcc${NC}"
        echo -e "${YELLOW}    CFLAGS_aarch64_unknown_nto_qnx800=-Vgcc_ntoaarch64le_cxx${NC}"
        echo -e "${YELLOW}    CXX_aarch64_unknown_nto_qnx800=qcc${NC}"
        echo -e "${YELLOW}    AR_aarch64_unknown_nto_qnx800=ntoaarch64-ar${NC}"
    fi
    echo -e "${YELLOW}    '${NC}"
    echo ""

if [ "$QNX_VERSION" = "7.1" ]; then
    echo -e "${YELLOW}# Build all targets (with std)${NC}"
        echo -e "${YELLOW}./x.py build --target $TARGETS library tools/rustfmt${NC}"
    else
        echo -e "${YELLOW}# Build host std library${NC}"
        echo -e "${YELLOW}./x.py build library${NC}"
        echo ""
        echo -e "${YELLOW}# Build QNX targets with core + alloc${NC}"
        echo -e "${YELLOW}./x.py build --target aarch64-unknown-nto-qnx800,x86_64-pc-nto-qnx800 library/core library/alloc${NC}"
        echo ""
        echo -e "${YELLOW}# Copy built libraries into single toolchain${NC}"
        echo -e "${YELLOW}# This is necessary because x.py is not automatically installing them together for some reason${NC}"
        echo -e "${YELLOW}cp -v $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1-std/x86_64-unknown-linux-gnu/release/deps/*.rlib \\${NC}"
        echo -e "${YELLOW}     $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1/lib/rustlib/x86_64-unknown-linux-gnu/lib/${NC}"
        echo -e "${YELLOW}cp -v $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1-std/x86_64-unknown-linux-gnu/release/deps/*.so \\${NC}"
        echo -e "${YELLOW}     $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1/lib/rustlib/x86_64-unknown-linux-gnu/lib/${NC}"
        echo -e "${YELLOW}cp -v $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1-std/x86_64-pc-nto-qnx800/release/deps/*.rlib \\${NC}"
        echo -e "${YELLOW}     $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1/lib/rustlib/x86_64-pc-nto-qnx800/lib/${NC}"
        echo -e "${YELLOW}cp -v $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1-std/aarch64-unknown-nto-qnx800/release/deps/*.rlib \\${NC}"
        echo -e "${YELLOW}     $RUSTDIR/build/x86_64-unknown-linux-gnu/stage1/lib/rustlib/aarch64-unknown-nto-qnx800/lib/${NC}"
    fi

else
    echo -e "${BLUE}Building Rust compiler...${NC}"
    echo ""
    
    COMPILER_START_TIME=$(date +%s)
    
    if [ "$QNX_VERSION" = "7.1" ]; then
        export build_env='
            CC_x86_64_pc_nto_qnx710=qcc
            CFLAGS_x86_64_pc_nto_qnx710=-Vgcc_ntox86_64_cxx
            CXX_x86_64_pc_nto_qnx710=qcc
            AR_x86_64_pc_nto_qnx710=ntox86_64-ar
            CC_aarch64_unknown_nto_qnx710=qcc
            CFLAGS_aarch64_unknown_nto_qnx710=-Vgcc_ntoaarch64le_cxx
            CXX_aarch64_unknown_nto_qnx710=qcc
            AR_aarch64_unknown_nto_qnx710=ntoaarch64-ar
        '
    elif [ "$QNX_VERSION" = "8.0" ]; then
        export build_env='
            CC_x86_64_pc_nto_qnx800=qcc
            CFLAGS_x86_64_pc_nto_qnx800=-Vgcc_ntox86_64_cxx
            CXX_x86_64_pc_nto_qnx800=qcc
            AR_x86_64_pc_nto_qnx800=ntox86_64-ar
            CC_aarch64_unknown_nto_qnx800=qcc
            CFLAGS_aarch64_unknown_nto_qnx800=-Vgcc_ntoaarch64le_cxx
            CXX_aarch64_unknown_nto_qnx800=qcc
            AR_aarch64_unknown_nto_qnx800=ntoaarch64-ar
        '
    fi
    
    if [ "$QNX_VERSION" = "7.1" ]; then
        # Build everything together (with std)
        set +e
        ./x.py build --target "$TARGETS" library tools/rustfmt
        COMPILER_EXIT_CODE=$?
        set -e
    else
        # Build libraries for host (std), then QNX targets (core/alloc only)
        # Build std library for host
        set +e
        ./x.py build library
        HOST_EXIT_CODE=$?
        set -e
        
        if [ $HOST_EXIT_CODE -ne 0 ]; then
            COMPILER_END_TIME=$(date +%s)
            COMPILER_DURATION=$((COMPILER_END_TIME - COMPILER_START_TIME))
            echo -e "${RED}✗ Host std build failed (${COMPILER_DURATION}s, exit code: $HOST_EXIT_CODE)${NC}"
            exit $HOST_EXIT_CODE
        fi
        echo -e "${GREEN}✓ Host std library built successfully${NC}"
        echo ""
        
        # Build QNX targets with only core and alloc
        set +e
        ./x.py build --target "$TARGETS" library/core library/alloc
        QNX_EXIT_CODE=$?
        set -e
        
        if [ $QNX_EXIT_CODE -ne 0 ]; then
            COMPILER_END_TIME=$(date +%s)
            COMPILER_DURATION=$((COMPILER_END_TIME - COMPILER_START_TIME))
            echo -e "${RED}✗ QNX targets build failed (${COMPILER_DURATION}s, exit code: $QNX_EXIT_CODE)${NC}"
            exit $QNX_EXIT_CODE
        fi
        echo -e "${GREEN}✓ QNX targets built successfully${NC}"
        echo ""
        
        # Copy built libraries into single toolchain
        # This is necessary because x.py is not automatically installing them together for some reason...
        STAGE1_STD_DIR="$RUSTDIR/build/x86_64-unknown-linux-gnu/stage1-std"
        STAGE1_DIR="$RUSTDIR/build/x86_64-unknown-linux-gnu/stage1"
        
        # Copy host std libraries
        if [ -d "$STAGE1_STD_DIR/x86_64-unknown-linux-gnu/release/deps" ]; then
            cp -v "$STAGE1_STD_DIR/x86_64-unknown-linux-gnu/release/deps"/*.rlib \
                 "$STAGE1_DIR/lib/rustlib/x86_64-unknown-linux-gnu/lib/" 2>/dev/null || true
            cp -v "$STAGE1_STD_DIR/x86_64-unknown-linux-gnu/release/deps"/*.so \
                 "$STAGE1_DIR/lib/rustlib/x86_64-unknown-linux-gnu/lib/" 2>/dev/null || true
            cp -v "$STAGE1_STD_DIR/x86_64-unknown-linux-gnu/release/deps"/*.rmeta \
                 "$STAGE1_DIR/lib/rustlib/x86_64-unknown-linux-gnu/lib/" 2>/dev/null || true
            echo -e "${GREEN}✓ Host std libraries copied to stage1 sysroot${NC}"
        else
            echo -e "${YELLOW}⚠ Host std libraries not found in expected location${NC}"
        fi
        
        # Copy QNX x86_64 libraries
        if [ -d "$STAGE1_STD_DIR/x86_64-pc-nto-qnx800/release/deps" ]; then
            cp -v "$STAGE1_STD_DIR/x86_64-pc-nto-qnx800/release/deps"/*.rlib \
                 "$STAGE1_DIR/lib/rustlib/x86_64-pc-nto-qnx800/lib/" 2>/dev/null || true
            echo -e "${GREEN}✓ QNX x86_64 libraries copied to stage1 sysroot${NC}"
        else
            echo -e "${YELLOW}⚠ QNX x86_64 libraries not found in expected location${NC}"
        fi
        
        # Copy QNX aarch64 libraries
        if [ -d "$STAGE1_STD_DIR/aarch64-unknown-nto-qnx800/release/deps" ]; then
            cp -v "$STAGE1_STD_DIR/aarch64-unknown-nto-qnx800/release/deps"/*.rlib \
                 "$STAGE1_DIR/lib/rustlib/aarch64-unknown-nto-qnx800/lib/" 2>/dev/null || true
            echo -e "${GREEN}✓ QNX aarch64 libraries copied to stage1 sysroot${NC}"
        else
            echo -e "${YELLOW}⚠ QNX aarch64 libraries not found in expected location${NC}"
        fi
        
        COMPILER_EXIT_CODE=$QNX_EXIT_CODE
    fi
    
    COMPILER_END_TIME=$(date +%s)
    COMPILER_DURATION=$((COMPILER_END_TIME - COMPILER_START_TIME))
    
    echo ""

    if [ $COMPILER_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ Rust compiler build completed successfully (${COMPILER_DURATION}s)${NC}"
    else
        echo -e "${RED}✗ Rust compiler build failed (${COMPILER_DURATION}s, exit code: $COMPILER_EXIT_CODE)${NC}"
        exit $COMPILER_EXIT_CODE
    fi

fi

echo ""

# Create Rustup Toolchain Link
# ----------------------------

STAGE1_DIR="$RUSTDIR/build/x86_64-unknown-linux-gnu/stage1"

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}# Create alias for built toolchain${NC}"
    echo -e "${YELLOW}rustup toolchain link $TOOLCHAIN_NAME $STAGE1_DIR${NC}"
else
    echo -e "${BLUE}Creating rustup toolchain link...${NC}"
    if rustup toolchain list | grep -q "^$TOOLCHAIN_NAME"; then
        rustup toolchain uninstall "$TOOLCHAIN_NAME"
    fi
    rustup toolchain link "$TOOLCHAIN_NAME" "$STAGE1_DIR"
    echo -e "${GREEN}✓ Toolchain '$TOOLCHAIN_NAME' linked to $STAGE1_DIR${NC}"
fi

echo ""

# Build Remote Testing Tools
# --------------------------

if [ "$BUILD_REMOTE_TOOLS" = true ]; then
    if [ "$QNX_VERSION" = "8.0" ]; then
        echo -e "${YELLOW}⚠ Skip remote testing tools build on QNX 8.0 (only supported on QNX 7.1)${NC}"
    else
        echo -e "${BLUE}Building remote testing tools...${NC}"
        
        if [ "$DRY_RUN" = true ]; then
            echo -e "${YELLOW}# Build remote-test-client${NC}"
            echo -e "${YELLOW}cd $RUSTDIR && cargo +$TOOLCHAIN_NAME build --release --package remote-test-client${NC}"
            echo -e "${YELLOW}cp $RUSTDIR/target/release/remote-test-client $TOOLCHAIN_BIN_DIR${NC}"
            
            echo -e "${YELLOW}# Build remote-test-server for QNX targets${NC}"
            echo -e "${YELLOW}cd $RUSTDIR && cargo +$TOOLCHAIN_NAME build --release --package remote-test-server --target x86_64-pc-nto-qnx710${NC}"
            echo -e "${YELLOW}cp $RUSTDIR/target/x86_64-pc-nto-qnx710/release/remote-test-server $SHARED_DIR/remote-test-server-x86_64${NC}"
            echo -e "${YELLOW}cd $RUSTDIR && cargo +$TOOLCHAIN_NAME build --release --package remote-test-server --target aarch64-unknown-nto-qnx710${NC}"
            echo -e "${YELLOW}cp $RUSTDIR/target/aarch64-unknown-nto-qnx710/release/remote-test-server $SHARED_DIR/remote-test-server-aarch64${NC}"
        else
            mkdir -p "$SHARED_DIR"
            
            # Build remote-test-client for host
            echo -e "${BLUE}Building remote-test-client...${NC}"
            CLIENT_START_TIME=$(date +%s)
            cd "$RUSTDIR"
            set +e
            cargo +"$TOOLCHAIN_NAME" build --release --package remote-test-client
            CLIENT_EXIT_CODE=$?
            set -e
            CLIENT_END_TIME=$(date +%s)
            CLIENT_DURATION=$((CLIENT_END_TIME - CLIENT_START_TIME))
            
            if [ $CLIENT_EXIT_CODE -eq 0 ]; then
                echo -e "${GREEN}✓ remote-test-client built successfully (${CLIENT_DURATION}s)${NC}"
                
                # Copy to toolchain bin directory
                CLIENT_SRC="$RUSTDIR/target/release/remote-test-client"
                if [ -f "$CLIENT_SRC" ]; then
                    if [ -d "$TOOLCHAIN_BIN_DIR" ]; then
                        cp "$CLIENT_SRC" "$TOOLCHAIN_BIN_DIR/"
                        echo -e "${GREEN}✓ remote-test-client copied to $TOOLCHAIN_BIN_DIR${NC}"
                    else
                        echo -e "${YELLOW}⚠ Toolchain bin directory not found: $TOOLCHAIN_BIN_DIR${NC}"
                        echo -e "${YELLOW}remote-test-client available at: $CLIENT_SRC${NC}"
                    fi
                else
                    echo -e "${RED}✗ remote-test-client binary not found at $CLIENT_SRC${NC}"
                fi
            else
                echo -e "${RED}✗ remote-test-client build failed (${CLIENT_DURATION}s, exit code: $CLIENT_EXIT_CODE)${NC}"
            fi
            
            # Build remote-test-server for QNX targets
            X86_TARGET="x86_64-pc-nto-qnx710"
            AARCH64_TARGET="aarch64-unknown-nto-qnx710"
            
            # Build for x86_64
            echo -e "${BLUE}Building remote-test-server for x86_64...${NC}"
            SERVER_START_TIME=$(date +%s)
            set +e
            cargo +"$TOOLCHAIN_NAME" build --release --package remote-test-server --target "$X86_TARGET"
            SERVER_EXIT_CODE=$?
            set -e
            SERVER_END_TIME=$(date +%s)
            SERVER_DURATION=$((SERVER_END_TIME - SERVER_START_TIME))
            
            if [ $SERVER_EXIT_CODE -eq 0 ]; then
                echo -e "${GREEN}✓ remote-test-server for x86_64 built successfully (${SERVER_DURATION}s)${NC}"
                SERVER_SRC="$RUSTDIR/target/$X86_TARGET/release/remote-test-server"
                if [ -f "$SERVER_SRC" ]; then
                    cp "$SERVER_SRC" "$SHARED_DIR/remote-test-server-x86_64"
                    echo -e "${GREEN}✓ remote-test-server-x86_64 copied to $SHARED_DIR${NC}"
                else
                    echo -e "${RED}✗ remote-test-server binary not found at $SERVER_SRC${NC}"
                fi
            else
                echo -e "${RED}✗ remote-test-server for x86_64 build failed (${SERVER_DURATION}s, exit code: $SERVER_EXIT_CODE)${NC}"
            fi
            
            # Build for aarch64
            echo -e "${BLUE}Building remote-test-server for aarch64...${NC}"
            SERVER_START_TIME=$(date +%s)
            set +e
            cargo +"$TOOLCHAIN_NAME" build --release --package remote-test-server --target "$AARCH64_TARGET"
            SERVER_EXIT_CODE=$?
            set -e
            SERVER_END_TIME=$(date +%s)
            SERVER_DURATION=$((SERVER_END_TIME - SERVER_START_TIME))
            
            if [ $SERVER_EXIT_CODE -eq 0 ]; then
                echo -e "${GREEN}✓ remote-test-server for aarch64 built successfully (${SERVER_DURATION}s)${NC}"
                SERVER_SRC="$RUSTDIR/target/$AARCH64_TARGET/release/remote-test-server"
                if [ -f "$SERVER_SRC" ]; then
                    cp "$SERVER_SRC" "$SHARED_DIR/remote-test-server-aarch64"
                    echo -e "${GREEN}✓ remote-test-server-aarch64 copied to $SHARED_DIR${NC}"
                else
                    echo -e "${RED}✗ remote-test-server binary not found at $SERVER_SRC${NC}"
                fi
            else
                echo -e "${RED}✗ remote-test-server for aarch64 build failed (${SERVER_DURATION}s, exit code: $SERVER_EXIT_CODE)${NC}"
            fi
        fi
    fi
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
echo -e "Rust Version: $RUST_VERSION"
if [ "$QNX_VERSION" = "8.0" ]; then
    echo -e "Build Configuration: no_std (core + alloc)"
fi
echo -e "Build Targets: $TARGETS"
echo -e "Toolchain Name: $TOOLCHAIN_NAME"
if [ "$BUILD_REMOTE_TOOLS" = true ]; then
    echo -e "Remote Tools: Built for both architectures"
fi
echo -e "Total execution time: ${TOTAL_DURATION}s"

if [ "$DRY_RUN" = false ]; then
    echo -e "${GREEN}🎉 Rust toolchain build completed!${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "  1. Use the toolchain: cargo +$TOOLCHAIN_NAME build --target <qnx-target>"
    echo -e "  2. Available QNX targets:"
    if [ "$QNX_VERSION" = "7.1" ]; then
        echo -e "     - x86_64-pc-nto-qnx710"
        echo -e "     - aarch64-unknown-nto-qnx710"
    else
        echo -e "     - x86_64-pc-nto-qnx800"
        echo -e "     - aarch64-unknown-nto-qnx800"
    fi
    if [ "$BUILD_REMOTE_TOOLS" = true ]; then
        echo -e "  3. Remote test servers available in: $SHARED_DIR"
        echo -e "     - remote-test-server-x86_64 (for x86_64 targets)"
        echo -e "     - remote-test-server-aarch64 (for aarch64 targets)"
        if [ -d "$TOOLCHAIN_BIN_DIR" ]; then
            echo -e "  4. Remote test client available at: $TOOLCHAIN_BIN_DIR/remote-test-client"
        else
            echo -e "  4. Remote test client available in src/tools/remote-test-client/target/release/"
        fi
    fi
else
    echo -e "${YELLOW}Dry run completed. No build was performed.${NC}"
fi
