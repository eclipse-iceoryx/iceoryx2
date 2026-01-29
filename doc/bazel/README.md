# Use iceoryx2 with Bazel

iceoryx2 supports Bazel as a build target, via both bazel workspaces and bazel
modules. Other operating systems are not yet supported.

## Setup with Bazel build system

You can pull the iceoryx2 repo into your bazel project in two different manners.
Full examples can be found under `examples/bazel`.

### Setup via Bzlmod (Recommended)

There's an example for using Bazel under `examples/bazel`. Alternatively, you
can follow this abbreviated guide, and ensure the following is present in your
`MODULE.bazel` or `MODULE` file:

```bazel
bazel_dep(name = "iceoryx2", version = "0.8.1")
bazel_dep(name = "rules_rust", version = "0.68.1")

# ==============================================================================
# Iceoryx2 Setup
# ==============================================================================


git_override(
    module_name = iceoryx2,
    remote = "https://github.com/eclipse-iceoryx/iceoryx2.git",
    # Insert your git ref below. It can be a tag, commit, or branch
    commit = "0.8.1"
)

# ==============================================================================
# Rust Setup (Example)
# ==============================================================================

rust = use_extension("@rules_rust//rust:extensions.bzl", "rust")
rust.toolchain(
    edition = "2021",
    versions = ["1.87.0"],
)
```

### Setup via Workspace (legacy)

Using iceoryx2 via Bazel Workspace is no longer supported. If you are still using
Bazel Workspaces, import iceoryx2 library with language-specific methods or
migrate to Bazel Modules (see [Bazel Migration Guide](https://bazel.build/external/migration)).

### Initial Build

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

## Linking iceoryx2 in Your `BUILD.bazel`

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

## Feature Flags

iceoryx2 provides several feature flags that can be configured using bazel
build options. These flags allow you to enable or disable specific features
when building the project.

| Feature Flag     | Valid Values | Default |
| ---------------- | ------------ | ------- |
| std              | on, off      | on      |
| dev_permissions  | on, off      | off     |
| logger_buffer    | on, off      | off     |
| logger_console   | on, off      | on      |
| logger_file      | on, off      | off     |
| logger_log       | on, off      | off     |
| logger_tracing   | on, off      | off     |


### Enabling a Feature Flag via Command Line

To set a feature flag directly from the command line during the build, use the
following format:

```bash
bazel build --@iceoryx2//:<feature_flag> //...
```

For example, to enable a feature flag called `foo`, you would run:

```bash
bazel build --@iceoryx2//:foo //...
```

### Setting Feature Flags in .bazelrc

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

## Running iceoryx2 Tests in External Project

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
