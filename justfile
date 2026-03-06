import '.just/common.just'
import '.just/build.just'
import '.just/test.just'
import '.just/bundle.just'

# Show available commands and usage examples
default:
    @echo "iceoryx2 development helpers"
    @echo ""

    @echo "Usage:"
    @echo "  just build workspace                        # Build all crates in the workspace (default features)"
    @echo "  just build tests                            # Build all tests in the workspace (standard framework, default features)"
    @echo "  just build tests --no_std [+toolchain]      # Build all no_std tests in the workspace (custom framework, no_std)"
    @echo "  just build <package>                        # Build specific package (default features)"
    @echo "  just build <package> --no_std [+toolchain]  # Build package (no_std)"
    @echo ""
    @echo "  just test workspace                         # Run tests in the workspace (standard framework, default features)"
    @echo "  just test workspace --no_std [+toolchain]   # Run all workspace tests (custom framework, no_std)"
    @echo "  just test <package>                         # Run tests for specific package (standard framework, default features)"
    @echo "  just test <package> --no_std [+toolchain]   # Run tests for specific package (custom framework, no_std)"
    @echo ""
    @echo "  just bundle tests --no_std [+toolchain] [--target=<triplet>] [--strip] [--compress]"
    @echo "                                              # Build all no_std tests in the workspace and bundle for deployment"
    @echo "                                              # +toolchain: Rust toolchain (e.g., +nightly, +stable)"
    @echo "                                              # --target=<triplet>: Target triplet (e.g., x86_64-unknown-linux-gnu)"
    @echo "                                              # --strip: Strip debug symbols from binaries"
    @echo "                                              # --compress: Create a compressed tarball"
    @echo ""
    @echo "Examples:"
    @echo "  just build workspace"
    @echo "  just build tests"
    @echo "  just build tests --no_std +nightly"
    @echo "  just build iceoryx2 --no_std"
    @echo "  just build iceoryx2-bb-elementary --no_std"
    @echo "  just test iceoryx2-bb-elementary --no_std"
    @echo "  just test iceoryx2-bb-elementary --no_std +nightly"
    @echo "  just bundle tests --no_std --strip --compress"
    @echo "  just bundle tests --no_std +nightly --target=aarch64-unknown-linux-gnu --strip"
    @echo "  just bundle tests --no_std --target=x86_64-pc-nto-qnx800 +qnx800 --compress"
    @echo ""
    @echo "Run 'just list' to see all available recipes"

# Build workspace or a specific package
build target *flags="":
    @just _build-dispatch "{{target}}" {{flags}}

# Run tests for workspace or a specific package
test target *flags="":
    @just _test-dispatch "{{target}}" {{flags}}

# Bundle tests for deployment
bundle target *flags="":
    @just _bundle-dispatch "{{target}}" {{flags}}

# List all available recipes
list:
    @just --list
