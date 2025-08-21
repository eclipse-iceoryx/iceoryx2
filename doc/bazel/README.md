# Use iceoryx2 with bazel

On Linux, iceoryx2 can be build with bazel. Other OSes are not yet supported.

## Instructions for iceoryx2 users

To use iceoryx2 as an external dependency, ensure the following setup is present
in your `WORKSPACE` file:

```bazel
# Load iceoryx2 rules
ICEORYX2_VERSION = "0248ea57d0c405383ab099e14293ed8be2d23dac"

http_archive(
    name = "iceoryx2",
    sha256 = "8844b229d2ba23597dfe17df7a3baabd086a62944534aa804d482a6e46bdf5b8",
    strip_prefix = "iceoryx2-{}".format(ICEORYX2_VERSION),
    urls = [
        "https://github.com/eclipse-iceoryx/iceoryx2/archive/{}.tar.gz".format(ICEORYX2_VERSION),
    ],
)


# Load iceoryx rules
ICEORYX_VERSION = "2.95.7"

maybe(
    name = "iceoryx",
    repo_rule = http_archive,
    sha256 = "82c4fe7507d1609e1275a04a3fe8278ae20620aa30e1eede63f96a9c23308ab6",
    strip_prefix = "iceoryx-{}".format(ICEORYX_VERSION),
    url = "https://github.com/eclipse-iceoryx/iceoryx/archive/v{}.tar.gz".format(ICEORYX_VERSION),
)

load("@iceoryx//bazel:load_repositories.bzl", "load_repositories")

load_repositories()

load("@iceoryx//bazel:setup_repositories.bzl", "setup_repositories")

setup_repositories()


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
        "@iceoryx2//:benchmarks/queue/Cargo.toml",
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
        "@iceoryx2//:iceoryx2-ffi/c/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/concurrency-sync/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/configuration/Cargo.toml",
        "@iceoryx2//:iceoryx2-pal/posix/Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()


# Load skylib rules
BAZEL_SKYLIB_VERSION = "1.7.1"

# Load skylib for custom build config
maybe(
    name = "bazel_skylib",
    repo_rule = http_archive,
    sha256 = "bc283cdfcd526a52c3201279cda4bc298652efa898b10b4db0837dc51652756f",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/{version}/bazel-skylib-{version}.tar.gz".format(version = BAZEL_SKYLIB_VERSION),
        "https://github.com/bazelbuild/bazel-skylib/releases/download/{version}/bazel-skylib-{version}.tar.gz".format(version = BAZEL_SKYLIB_VERSION),
    ],
)

load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")

bazel_skylib_workspace()
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

To make syncing dependencies automatic for every build, add the
following line to your `.bazelrc` file:

```bazel
build --action_env=CARGO_BAZEL_REPIN=true
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

### Feature Flags

iceoryx2 provides several feature flags that can be configured using bazel
build options. These flags allow you to enable or disable specific features
when building the project.

#### Enabling a Feature Flag via Command Line

To set a feature flag directly from the command line during the build, use the
following format:

```bash
bazel build --@iceoryx2//:<feature_flag> //...
```

For example, to enable a feature flag called `foo`, you would run:

```bash
bazel build --@iceoryx2//:foo //...
```

#### Setting Feature Flags in .bazelrc

You can also persist feature flag configurations by specifying them in the
`.bazelrc` file. This method is useful for keeping your build settings
consistent across different environments.

```bazel
build --@iceoryx2//:<feature_flag>=on
```

For instance, to enable the `foo` feature by default in `.bazelrc`, you would add:

```bazel
build --@iceoryx2//:foo=on
```

#### List of Available Features

| Feature Flag            | Valid Values                 | Crate Default      |
| ----------------------- | ---------------------------- | ------------------ |
| dev_permissions         | auto, on, off                | auto == off        |
| logger_log              | auto, on, off                | auto == off        |
| logger_tracing          | auto, on, off                | auto == off        |

### Running iceory2x Tests in External Project

In general, the iceoryx2 tests can be run in parallel. However, there are
exceptions, as some tests deliberately try to bring the system into an
inconsistent state. When these tests are executed in parallel, they can become
flaky and may fail depending on which other tests are running concurrently.

To mitigate this, it is sufficient to prevent other tests from the same file
from running in parallel. This can be achieved by setting the following
environment variable in your .bashrc:

```bazel
test --action_env=RUST_TEST_THREADS=1
```

Assuming there are two test binaries, without the environment variable, all
tests would be executed in parallel, as illustrated below:

```ascii
bazel test /...
   |
   +--------------------+
   |                    |
  test-binary-A        test-binary-B
   |                    |
   +----------+         +----------+
   |          |         |          |
  test-A-1   test-A2   test-B-1   test-B2
   |          |         |          |
   +----------+         +----------+
   |                    |
   +--------------------+
   |
  test result
```

With the environment variable set, the test execution is partially serialized,
as shown below:

```ascii
bazel test /...
   |
   +--------------------+
   |                    |
  test-binary-A        test-binary-B
   |                    |
  test-A-1             test-B-1
   |                    |
  test-A-2             test-B-2
   |                    |
   +--------------------+
   |
  test result
```

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

### Common Pitfalls

1. **Handling `iceoryx2-ffi-c-cbindgen` Target**:

The `iceoryx2-ffi-c-cbindgen` target requires access to all crates listed in the
root `Cargo.toml`. Due to the sandboxed nature of Bazel builds, this can cause
issues if not properly configured.

1. **Managing Source Files**:

Every `BUILD.bazel` file includes an `all_srcs` filegroup to manage the source files
within the sandbox. The root `BUILD.bazel` file has an `all_srcs` filegroup that
references all sub-packages. When a new package is added, it must also be included
in this `all_srcs` filegroup.

1. **Not All Environment Variables are Available with Bazel**

`bazel` does not automatically export some environment variables that are
typically available with `cargo`, such as `CARGO_PKG_VERSION`. In these cases,
you will need to either set the environment variable manually in your `bazel`
configuration or find an appropriate workaround.
