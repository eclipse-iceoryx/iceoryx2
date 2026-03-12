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
_setup-dispatch target:
    #!/usr/bin/env bash
    case "{{target}}" in
        all ) just _setup-all ;;
        script-dependencies) just _setup-script-dependencies ;;
        *) echo "Unknown setup task: {{target}}"; exit 1 ;;
    esac

[private]
_setup-all:
    just _setup-script-dependencies

[private]
_setup-script-dependencies:
    #!/usr/bin/env bash

    _install_rust_script() {
        cargo install rust-script
        echo "✓ rust-script installed"
    }

    _install_jq() {
        if command -v apt &> /dev/null; then
            sudo apt install -y jq
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm jq
        elif command -v brew &> /dev/null; then
            brew install jq
        else
            echo "✗ could not install jq: no supported package manager found"
            echo "  Please install jq manually: https://jqlang.org/download/"
            exit 1
        fi
        echo "✓ jq installed"
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

        if command -v "$name" &> /dev/null; then
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

    if _prompt_install "rust-script"; then
        _install_rust_script
    fi

    if _prompt_install "jq"; then
        _install_jq
    fi

    if _prompt_install "markdownlint"; then
        _install_markdownlint
    fi

    echo "✓ Setup complete"
