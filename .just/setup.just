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
            echo "  all                    setup all dependencies"
            echo "  dev-tools              setup development tools (nextest, ...)"
            echo "  script-dependencies    setup just script dependencies"
            exit 1
            ;;
        all) just _setup-all ;;
        script-dependencies) just _setup-script-dependencies ;;
        testing-dependencies) just _setup-testing-dependencies ;;
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
