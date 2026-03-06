# =============================================================================
# Build Recipes
# =============================================================================

[private]
_build-dispatch target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-build-flags {{flags}})

    case "{{target}}" in
        workspace)
            just _build-workspace "$TOOLCHAIN" "$HAS_NOSTD"
            ;;
        tests)
            just _build-tests "$TOOLCHAIN" "$HAS_NOSTD" "$TARGET_TRIPLET"
            ;;
        *)
            just _build-package "{{target}}" "$TOOLCHAIN" "$HAS_NOSTD" "$TARGET_TRIPLET"
            ;;
    esac

[private]
_build-workspace toolchain has_nostd:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: entire workspace cannot be built with --no_std" >&2
        echo "Use 'just build <package> --no_std' to build compatible packages without std" >&2
        exit 1
    fi

    cargo {{toolchain}} build --workspace

[private]
_build-tests toolchain has_nostd target_triplet:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        NOSTD_CRATES=$(just _discover-nostd-test-crates)

        CRATE_COUNT=$(echo "$NOSTD_CRATES" | wc -l)
        echo ""
        echo "=========================================="
        echo "Building $CRATE_COUNT no_std test crates"
        echo "=========================================="
        echo ""

        BUILT_COUNT=0
        BUILT_CRATES=()
        for crate in $NOSTD_CRATES; do
            echo "[$((BUILT_COUNT + 1))/$CRATE_COUNT] Building $crate..."
            just _build-package "$crate" "{{toolchain}}" "true" "{{target_triplet}}"
            BUILT_COUNT=$((BUILT_COUNT + 1))
            BUILT_CRATES+=("$crate")
        done

        echo ""
        echo "=========================================="
        echo "Build Summary"
        echo "=========================================="
        echo "Successfully built $BUILT_COUNT no_std test crates:"
        for crate in "${BUILT_CRATES[@]}"; do
            echo "  ✓ $crate"
        done
        echo ""
    else
        cargo {{toolchain}} build --workspace --tests
    fi

[private]
_build-package package toolchain has_nostd target_triplet:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        else
            HOST_TARGET=$(just _detect-host-target "{{toolchain}}")
            TARGET_ARG="--target $HOST_TARGET"
        fi

        if [[ "{{package}}" == *"-tests-nostd" ]]; then
            RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C panic=abort" \
                cargo {{toolchain}} build \
                    --package {{package}} \
                    --no-default-features \
                    -Zbuild-std=core,alloc \
                    $TARGET_ARG
        else
            cargo {{toolchain}} build --package {{package}} --no-default-features
        fi
    else
        cargo {{toolchain}} build --package {{package}}
    fi
