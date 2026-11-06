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
[no-exit-message]
_setup-dispatch what:
    #!/usr/bin/env bash
    case "{{what}}" in
        "")
            echo "Usage: just setup <what>"
            echo ""
            echo "Groups (interactive):"
            echo "  all                         every group below"
            echo "  dev-dependencies            development tools (flatc)"
            echo "  script-dependencies         just script dependencies (rust-script, jq)"
            echo "  testing-dependencies        testing tools (cargo-nextest)"
            echo ""
            echo "Tools:"
            echo "  flatc                       the flatbuffers compiler"
            echo ""
            echo "ROS 2 workspace:"
            echo "  integrations-ros2           install the colcon build tools for cargo packages"
            echo "  integrations-ros2-messages  build the ros2 message crates from source (for installations not shipping them)"
            echo "  integrations-ros2-examples  build the ros2 examples (requires a sourced ROS 2 env)"
            exit 1
            ;;
        all) just _setup-all ;;
        script-dependencies) just _setup-script-dependencies ;;
        testing-dependencies) just _setup-testing-dependencies ;;
        dev-dependencies) just _setup-dev-dependencies ;;
        flatc) just _setup-flatc ;;
        integrations-ros2) just _setup-integrations-ros2 ;;
        integrations-ros2-messages) just _setup-integrations-ros2-messages ;;
        integrations-ros2-examples) just _setup-integrations-ros2-examples ;;
        *) echo "Unknown setup task: {{what}}"; exit 1 ;;
    esac

[private]
[no-exit-message]
_setup-all:
    just _setup-script-dependencies
    just _setup-testing-dependencies
    just _setup-dev-dependencies

[private]
[no-exit-message]
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
_setup-dev-dependencies:
    #!/usr/bin/env bash

    _prompt_install() {
        local name=$1
        local version=$2

        if which "$name" &> /dev/null && [ "$($name --version)" = "$name version $version" ]; then
            echo "✓ $name $version already installed, skipping"
            return 1
        fi

        read -p "Install $name $version? [y/N] " confirm
        if [[ "$confirm" == [yY] ]]; then
            return 0
        else
            echo "  Skipping $name"
            return 1
        fi
    }

    if _prompt_install "flatc" "$(just _flatbuffers-version)"; then
        just _setup-flatc
    fi

# Installs the flatc release binary of the flatbuffers crate version into
# `~/.local/bin`.
[private]
[no-exit-message]
_setup-flatc:
    #!/usr/bin/env bash
    set -euo pipefail

    VERSION=$(just _flatbuffers-version)
    if command -v flatc &> /dev/null && [ "$(flatc --version)" = "flatc version ${VERSION}" ]; then
        echo "✓ flatc ${VERSION} already installed, skipping"
        exit 0
    fi

    just _download-flatc "$VERSION" "${HOME}/.local/bin"
    echo "✓ flatc ${VERSION} installed to ${HOME}/.local/bin"

# Downloads the flatc release binary of flatbuffers `version` into
# `destination`.
[private]
[no-exit-message]
_download-flatc version destination:
    #!/usr/bin/env bash
    set -euo pipefail

    RELEASE="https://github.com/google/flatbuffers/releases/tag/v{{version}}"
    if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
        echo "Error: no flatc release binary for $(uname -s) $(uname -m) is downloaded by this recipe" >&2
        echo "  Install flatc {{version}} from ${RELEASE}" >&2
        exit 1
    fi

    ARCHIVE=$(mktemp -d)
    trap 'rm -rf "$ARCHIVE"' EXIT
    curl -sSfL -o "${ARCHIVE}/flatc.zip" \
        "https://github.com/google/flatbuffers/releases/download/v{{version}}/Linux.flatc.binary.clang++-18.zip"
    mkdir -p "{{destination}}"
    unzip -q -o -d "{{destination}}" "${ARCHIVE}/flatc.zip"

[private]
[no-exit-message]
_setup-integrations-ros2:
    just _setup-integrations-ros2-tools
    just _setup-integrations-ros2-schemas

# Installs the cargo-aware colcon build tools.
[private]
[no-exit-message]
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

# Generates the binary flatbuffer schemas the examples embed.
[private]
[no-exit-message]
_setup-integrations-ros2-schemas:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v flatc &> /dev/null; then
        echo "Error: flatc not found" >&2
        echo "  Install the FlatBuffers package, see integrations/ros2/README.md" >&2
        exit 1
    fi

    SCHEMAS="{{INTEGRATIONS_ROS2_DIR}}/examples/flatbuffer_translator/schemas"
    flatc -o "${SCHEMAS}" --schema --binary "${SCHEMAS}"/*.fbs

# Imports and builds the message crates generated by `rosidl_generator_rs`.
[private]
[no-exit-message]
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
        "--packages-up-to std_msgs geometry_msgs sensor_msgs example_interfaces rosidl_generator_rs"

    echo "✓ ros2 message crates built at {{INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE}}"

# Builds the examples as ROS 2 packages.
[private]
_setup-integrations-ros2-examples:
    #!/usr/bin/env bash
    set -eo pipefail

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
