# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

# =============================================================================
# Setup Recipes
# =============================================================================

[private]
_setup-dispatch what:
    #!/usr/bin/env bash
    case "{{what}}" in
        "")
            echo "Usage: just setup <what>"
            echo ""
            echo "  all                         setup all dependencies"
            echo "  testing-dependencies        setup testing tools (nextest, ...)"
            echo "  script-dependencies         setup just script dependencies"
            echo "  integrations-ros2           setup the ros2 environment and examples (all of the below)"
            echo "  integrations-ros2-tools     install the colcon build tools for cargo packages"
            echo "  integrations-ros2-messages  build the ros2 message crates (requires a sourced ROS 2 env)"
            echo "  integrations-ros2-examples  build the ros2 examples (requires the message crates)"
            exit 1
            ;;
        all) just _setup-all ;;
        script-dependencies) just _setup-script-dependencies ;;
        testing-dependencies) just _setup-testing-dependencies ;;
        integrations-ros2) just _setup-integrations-ros2 ;;
        integrations-ros2-tools) just _setup-integrations-ros2-tools ;;
        integrations-ros2-messages) just _setup-integrations-ros2-messages ;;
        integrations-ros2-examples) just _setup-integrations-ros2-examples ;;
        *) echo "Unknown setup task: {{what}}"; exit 1 ;;
    esac

[private]
_setup-all:
    just _setup-script-dependencies
    just _setup-testing-dependencies

[private]
_setup-script-dependencies:
    #!/usr/bin/env bash

    _install_rust_script() {
        cargo install rust-script
        echo "✓ rust-script installed"
    }

    _install_jq() {
        if which apt &> /dev/null; then
            sudo apt install -y jq
        elif which pacman &> /dev/null; then
            sudo pacman -S --noconfirm jq
        elif which brew &> /dev/null; then
            brew install jq
        else
            echo "✗ could not install jq: no supported package manager found"
            echo "  Please install jq manually: https://jqlang.org/download/"
            exit 1
        fi
        echo "✓ jq installed"
    }

    _install_rustfilt() {
        cargo install rustfilt
        echo "✓ rustfilt installed"
    }

    _install_grcov() {
        cargo install grcov
        echo "✓ grcov installed"
    }

    _install_gcovr() {
        if ! which pip &> /dev/null; then
            echo "✗ could not install gcovr: pip not found"
            echo "  Please install pip or gcovr manually: https://gcovr.com/en/stable/installation.html"
            exit 1
        fi
        read -p "Install gcovr with sudo? [y/N] " use_sudo
        if [[ "$use_sudo" == [yY] ]]; then
            sudo pip install gcovr
        else
            pip install gcovr
        fi
        echo "✓ gcovr installed"
    }

    _install_markdownlint() {
        read -p "Install markdownlint-cli with sudo? [y/N] " use_sudo
        if [[ "$use_sudo" == [yY] ]]; then
            sudo npm install -g markdownlint-cli
        else
            npm install -g markdownlint-cli
        fi
        echo "✓ markdownlint-cli installed"
    }

    _prompt_install() {
        local name=$1

        if which "$name" &> /dev/null; then
            echo "✓ $name already installed, skipping"
            return 1
        fi

        read -p "Install $name? [y/N] " confirm
        if [[ "$confirm" == [yY] ]]; then
            return 0
        else
            echo "  Skipping $name"
            return 1
        fi
    }

    for tool in rust-script jq rustfilt grcov gcovr markdownlint; do
        install_fn="_install_${tool//-/_}"
        if _prompt_install "$tool"; then "$install_fn"; fi
    done

[private]
_setup-testing-dependencies:
    #!/usr/bin/env bash

    _install_cargo_nextest() {
        cargo install cargo-nextest
        echo "✓ cargo-nextest installed"
    }

    _prompt_install() {
        local name=$1

        if which "$name" &> /dev/null; then
            echo "✓ $name already installed, skipping"
            return 1
        fi

        read -p "Install $name? [y/N] " confirm
        if [[ "$confirm" == [yY] ]]; then
            return 0
        else
            echo "  Skipping $name"
            return 1
        fi
    }

    if _prompt_install "cargo-nextest"; then
        _install_cargo_nextest
    fi

[private]
_setup-integrations-ros2:
    just _setup-integrations-ros2-tools
    just _setup-integrations-ros2-messages
    just _setup-integrations-ros2-examples

# Installs the cargo-aware colcon build tools.
[private]
_setup-integrations-ros2-tools:
    #!/usr/bin/env bash
    set -euo pipefail

    _pip_flags() {
        # An externally managed system Python refuses pip installs. Override
        # only if inside a container.
        local stdlib
        stdlib=$(python3 -c 'import sysconfig; print(sysconfig.get_path("stdlib"))')
        if [ ! -f "${stdlib}/EXTERNALLY-MANAGED" ]; then
            return
        fi
        if [ -f /run/.containerenv ] || [ -f /.dockerenv ]; then
            echo "--break-system-packages"
            return
        fi
        echo "Error: the system Python is externally managed" >&2
        echo "  Install the tools inside the development box, see integrations/ros2/README.md" >&2
        exit 1
    }

    PIP_FLAGS=$(_pip_flags)
    pip3 install $PIP_FLAGS colcon-cargo colcon-ros-cargo

    if ! command -v cargo-ament-build &> /dev/null; then
        cargo install cargo-ament-build
    fi

# Imports and builds the message crates generated by `rosidl_generator_rs`,
# which the older distributions do not ship.
[private]
_setup-integrations-ros2-messages:
    #!/usr/bin/env bash
    set -euo pipefail

    just _ros2-require-sourced

    REPOS="{{INTEGRATIONS_ROS2_MESSAGES_WORKSPACE}}/${ROS_DISTRO}.repos"
    if [ ! -f "${REPOS}" ]; then
        echo "Error: no repository set for '${ROS_DISTRO}' at ${REPOS}" >&2
        exit 1
    fi

    mkdir -p "{{INTEGRATIONS_ROS2_MESSAGES_SOURCES}}"
    vcs import --skip-existing "{{INTEGRATIONS_ROS2_MESSAGES_SOURCES}}" < "${REPOS}"

    just _setup-integrations-ros2-colcon-build \
        "{{INTEGRATIONS_ROS2_MESSAGES_TARGET_DIR}}" \
        "{{INTEGRATIONS_ROS2_MESSAGES_TARGET_DIR}}" \
        "--packages-up-to std_msgs geometry_msgs rosidl_generator_rs"

    echo "✓ ros2 message crates built at {{INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE}}"

# Builds the examples on top of the message crates.
[private]
_setup-integrations-ros2-examples:
    #!/usr/bin/env bash
    set -eo pipefail

    just _ros2-require-messages
    source "{{INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE}}/setup.bash"

    PACKAGE="demo_nodes_iceoryx2"

    just _setup-integrations-ros2-colcon-build \
        "{{INTEGRATIONS_ROS2_EXAMPLES_WORKSPACE}}" \
        "{{INTEGRATIONS_ROS2_EXAMPLES_TARGET_DIR}}" \
        "--packages-select ${PACKAGE}"

    echo "✓ ros2 examples built at {{INTEGRATIONS_ROS2_EXAMPLES_INSTALL_BASE}}"

# Builds `workspace` with colcon from its root, placing everything colcon
# writes under `target`.
[private]
_setup-integrations-ros2-colcon-build workspace target *args:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "{{workspace}}"
    colcon --log-base "{{target}}/log" build \
        --build-base "{{target}}/build" \
        --install-base "{{target}}/install" \
        {{args}}
