# Frequently Asked Questions - iceoryx Developer Edition

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
