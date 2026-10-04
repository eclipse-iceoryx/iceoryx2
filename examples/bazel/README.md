# iceoryx2 Bazel Example

This directory contains a standalone Bazel example demonstrating how to use
iceoryx2 with Bazel modules (bzlmod) and `rules_rs`. Bazel 7.5.0 or newer is
supported with the included compatibility patches. Cargo dependencies are
resolved directly from iceoryx2's `Cargo.lock`; no repin step is needed.

## Quick Start

### Build Everything

From this directory:

```bash
bazel build //...
```

### Run Rust Examples

Terminal 1 (subscriber):

```bash
bazel run //rust:subscriber
```

Terminal 2 (publisher):

```bash
bazel run //rust:publisher
```

### Run C++ Examples

Terminal 1 (subscriber):

```bash
bazel run //cxx:example_subscriber_cpp
```

Terminal 2 (publisher):

```bash
bazel run //cxx:example_publisher_cpp
```

## Using in Your Own Project

To use iceoryx2 in your own Bazel project:

The two patch files in this directory are symlinks to `../../bazel`. When copying
this example, copy the actual patch files into your project so the root module's
`single_version_override` declarations can resolve them. Keep
`build --experimental_cc_static_library` in `.bazelrc` when using Bazel 7.

1. **Copy this directory** as a starting template
2. **Update MODULE.bazel**:
   * Change `local_path_override` to point to your iceoryx2 location, or
   * Use `git_override` for a specific commit:
     ```python
     git_override(
         module_name = "iceoryx2",
         remote = "https://github.com/eclipse-iceoryx/iceoryx2.git",
         commit = "<commit>", # or whatever release you'd like to use
     )
     ```

3. **Link against iceoryx2** in your BUILD.bazel files:
   * Rust: `"@iceoryx2//:iceoryx2"`
   * C++: `"@iceoryx2//:iceoryx2-cxx-static"` or `"@iceoryx2//:iceoryx2-cxx-shared"`
   * C: `"@iceoryx2//:iceoryx2-c-static"` or `"@iceoryx2//:iceoryx2-c-shared"`
