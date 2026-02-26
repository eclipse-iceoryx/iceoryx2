# =============================================================================
# Utility Recipes
# =============================================================================

[private]
_parse-build-flags *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    TOOLCHAIN=""
    HAS_NOSTD="false"
    TARGET_TRIPLET=""

    for arg in {{flags}}; do
        if [[ "$arg" == +* ]]; then
            TOOLCHAIN="$arg"
        elif [[ "$arg" == "--no_std" ]]; then
            HAS_NOSTD="true"
        elif [[ "$arg" == --target=* ]]; then
            TARGET_TRIPLET="${arg#--target=}"
        fi
    done

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"

[private]
_parse-bundle-flags *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    TOOLCHAIN=""
    TARGET_TRIPLET=""
    DO_STRIP="false"
    DO_COMPRESS="false"
    HAS_NOSTD="false"

    for arg in {{flags}}; do
        if [[ "$arg" == +* ]]; then
            TOOLCHAIN="$arg"
        elif [[ "$arg" == --target=* ]]; then
            TARGET_TRIPLET="${arg#--target=}"
        elif [[ "$arg" == "--strip" ]]; then
            DO_STRIP="true"
        elif [[ "$arg" == "--compress" ]]; then
            DO_COMPRESS="true"
        elif [[ "$arg" == "--no_std" ]]; then
            HAS_NOSTD="true"
        fi
    done

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"
    echo "DO_STRIP='$DO_STRIP'"
    echo "DO_COMPRESS='$DO_COMPRESS'"
    echo "HAS_NOSTD='$HAS_NOSTD'"

[private]
_discover-nostd-test-crates:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v jq &> /dev/null; then
        echo "Error: jq is required but not found on your system" >&2
        echo "" >&2
        echo "Please install jq to use no_std test commands:" >&2
        echo "  • Arch Linux:    sudo pacman -S jq" >&2
        echo "  • Ubuntu/Debian: sudo apt install jq" >&2
        echo "  • macOS:         brew install jq" >&2
        echo "" >&2
        exit 1
    fi

    cargo metadata --format-version=1 --no-deps \
        | jq -r '.packages[].name' \
        | grep -- '-tests-nostd$' \
        | sort

[private]
_detect-host-target toolchain:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -n "{{toolchain}}" ]; then
        rustc {{toolchain}} -vV | sed -n 's|host: ||p'
    else
        rustc -vV | sed -n 's|host: ||p'
    fi
