# Frequently Asked Questions - iceoryx Developer Edition

## Set `LogLevel::Trace` as default

During development, a detailed log output is often required. Therefore, the log
level should default to `LogLevel::Trace`. All iceoryx2 examples use the call
`set_log_level_from_env_or(LogLevel::Info)` to allow overriding the default.

Users can override the log level in the following ways:

1. **Environment variable**
   Set the variable in the terminal:
   ```bash
   export IOX2_LOG_LEVEL="Trace"
   ```

2. **Cargo environment configuration**
   Define `IOX2_LOG_LEVEL` in `$GITROOT$/.cargo/config.toml` or globally in
   `~/.cargo/config.toml` by adding:

   ```toml
   [env]
   IOX2_LOG_LEVEL = "Trace"
   ```

## Tests marked with `#[should_panic]` attribute do not panic in release builds

This usually happens when the panic is triggert via a `debug_assert` macro.
This macro is not active when the `-C debug-assertions` flag is not set, which
is the case for release builds.
To fix this problem, just add a `#[cfg(debug_assertions)]` to the test.

```rs
#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn accessing_uninitialized_foo_fails() {
    // ...
}
```

## The bazel build fails with an error mentioning `crate_index`, `manifest` and `Cargo.toml`

The error looks similar to this:

```ascii
An error occurred during the fetch of repository 'crate_index'
...
Error computing the main repository mapping: no such package '@@crate_index//'
...
Error: Some manifests are not being tracked. Please add the following labels to the `manifests` key: {
    "//iceoryx2-foo/bar:Cargo.toml",
}
```

It seems a new crate is added to the root `Cargo.toml` and bazel is complaining
that it is not added to the `WORKSPACE.bazel` file for the `crate_index` target.

## The bazel build fails to find a Cargo.toml file for a newly added crate

Assuming a new crate `bar` located in directory `iceoryx2-foo`, the end of the
error message will be in the form:

```ascii
Caused by:
        failed to read `/home/runner/.bazel/sandbox/processwrapper-sandbox/2/execroot/_main/iceoryx2-foo/bar/Cargo.toml`\n\nCaused by:\n  No such file or directory (os error 2)\n"
```

This is fixed by adding the new crate to the `filegroup` in `BUILD.bazel` in
the project root.

## The bazel build fails to build the crate index

An error in this form:

```ascii
--stderr:
thread 'main' panicked at external/crate_index__ring-0.17.14/build.rs:287:43:
called `Option::unwrap()` on a `None` value
```

may be due to an out-of-date `rules_rust` being used by the bazel build.
Updating to a newer version may resolve the issue.
