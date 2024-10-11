# Use iceoryx2 with bazel

On Linux, iceoryx2 can be build with bazel. Other OSes are not yet supported.

## Instructions for iceoryx2 users

To use iceoryx2 as an external dependency, ensure the following setup is present
in your `WORKSPACE` file:

```bazel
ICEORYX2_VERSION = "branch-tag-or-commit-hash"

http_archive(
    name = "iceoryx2",
    sha256 = "add-the-correct-sha256-sum",
    strip_prefix = "iceoryx2-{}".format(ICEORYX2_VERSION),
    urls = [
        "https://github.com/eclipse-iceoryx/iceoryx2/archive/{}.tar.gz".format(ICEORYX2_VERSION),
    ],
)


# Load Rust rules
# Use v0.26 to support bazel v6.2
maybe(
    name = "rules_rust",
    repo_rule = http_archive,
    sha256 = "9d04e658878d23f4b00163a72da3db03ddb451273eb347df7d7c50838d698f49",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.26.0/rules_rust-v0.26.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()
rust_register_toolchains(
    edition = "2021",
    versions = [
        "1.80.0"
    ],
)


# Load prebuilt bindgen
maybe(
    name = "bindgen",
    repo_rule = http_archive,
    sha256 = "b7e2321ee8c617f14ccc5b9f39b3a804db173ee217e924ad93ed16af6bc62b1d",
    strip_prefix = "bindgen-cli-x86_64-unknown-linux-gnu",
    urls = ["https://github.com/rust-lang/rust-bindgen/releases/download/v0.69.5/bindgen-cli-x86_64-unknown-linux-gnu.tar.xz"],
    build_file_content = """
filegroup(
    name = "bindgen-cli",
    srcs = ["bindgen"],
    visibility = ["//visibility:public"],
)
    """,
)

# Load prebuilt cbindgen
maybe(
    name = "cbindgen",
    repo_rule = http_file,
    sha256 = "521836d00863cb129283054e5090eb17563614e6328b7a1610e30949a05feaea",
    urls = ["https://github.com/mozilla/cbindgen/releases/download/0.26.0/cbindgen"],
    executable = True,
)

# Load external crates
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

maybe(
    name = "crate_index",
    repo_rule = crates_repository,
    cargo_lockfile = "@iceoryx2//:Cargo.lock",
    lockfile = "@iceoryx2//:Cargo.Bazel.lock",
    manifests = [
        "@iceoryx2//:Cargo.toml",
        "@iceoryx2//:benchmarks/event/Cargo.toml",
        "@iceoryx2//:benchmarks/publish-subscribe/Cargo.toml",
        "@iceoryx2//:examples/Cargo.toml",
        "@iceoryx2//:iceoryx2/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/container/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/derive-macros/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/elementary/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/lock-free/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/log/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/memory/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/posix/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/system-types/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/testing/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/threadsafe/Cargo.toml",
        "@iceoryx2//:iceoryx2-bb/trait-tests/Cargo.toml",
        "@iceoryx2//:iceoryx2-cal/Cargo.toml",
        "@iceoryx2//:iceoryx2-cli/Cargo.toml",
        "@iceoryx2//:iceoryx2-ffi/ffi-macros/Cargo.toml",
        "@iceoryx2//:iceoryx2-ffi/ffi/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/concurrency-sync/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/configuration/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/posix/Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()
```

### Syncing Dependencies

For the initial build, you must sync dependencies with crates.io.
This can be done by running one of the following commands:

```bash
CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index
```

or

```bash
CARGO_BAZEL_REPIN=1 bazel build //...
```

For more details, refer to the
[Crate Universe Repinning Guide](https://bazelbuild.github.io/rules_rust/crate_universe.html#repinning--updating-dependencies-1).

### Linking iceoryx2 in Your `BUILD.bazel`

To use iceoryx2 in your Bazel project, link the appropriate static or shared
library. For example, in `BUILD.bazel`:

```bazel
load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library")

cc_binary(
    name = "main",
    srcs = [
        "main.cpp",
    ],
    deps = [
        "@iceoryx2//:iceoryx2-cxx-static",
    ],
)
```

* **For Rust**: Use `@iceoryx2//:iceoryx2`
* **For C**: Use `@iceoryx2//:iceoryx2-c-static` or
  `@iceoryx2//:iceoryx2-c-shared`
* **For C++**: Use `@iceoryx2//:iceoryx2-cxx-static` or
  `@iceoryx2//:iceoryx2-cxx-shared`

## Instructions for iceoryx2 Developers

When working with Bazel and Cargo in this project, ensure the following steps are
followed to maintain consistency between both build systems:

### Adding Crates to Targets

1. **In `Cargo.toml`**: When a new crate is added to a target's `Cargo.toml`
file, the same crate must also be referenced in the corresponding target
within the `BUILD.bazel` file.

1. **In `WORKSPACE.bazel`**: If a new crate is added to the root `Cargo.toml`
file, it must also be included in the `crate_index` target located in the
`WORKSPACE.bazel` file.

### Updating Dependencies

Any time a dependency is added or changed, the `Cargo.Bazel.lock` file must be
updated by running:

```bash
CARGO_BAZEL_REPIN=1 bazel build //...
```

### Common Pitfalls

1. **Handling `iceoryx2-ffi-cbindgen` Target**:

The `iceoryx2-ffi-cbindgen` target requires access to all crates listed in the
root `Cargo.toml`. Due to the sandboxed nature of Bazel builds, this can cause
issues if not properly configured.

1. **Managing Source Files**:

Every `BUILD.bazel` file includes an `all_srcs` filegroup to manage the source files
within the sandbox. The root `BUILD.bazel` file has an `all_srcs` filegroup that
references all sub-packages. When a new package is added, it must also be included
in this `all_srcs` filegroup.
