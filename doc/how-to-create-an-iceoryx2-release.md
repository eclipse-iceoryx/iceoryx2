# How To Create An iceoryx2 Release

1. Create new release branch via the GitHub web interface and name it
   `release_X.Y.Z`
2. `just prepare-release all versions --version X.Y.Z`
3. `cargo build --workspace --all-targets` refresh cargo lock file
4. `USE_BAZEL_VERSION=7.4.1 bazel build //...` refresh bazel lock file
5. `peotry --project iceoryx2-ffi/python build-into-venv` refresh poetry lock file

There are three scripts to perform the iceoryx2 release

* internal/scripts/release/release_preparation.sh
* internal/scripts/release/release_tagging.sh
* internal/scripts/release/release_publish.sh

Each script has a `howto` parameter which prints the steps that are performed.

The `release_preparation.sh` script needs to be called with `--new-version x.y.z`.
All the other scripts can be called without arguments.

After the `release_preparation.sh` script is run, a PR needs to be created and
merged to the main branch. Once merged, the `release_tagging.sh` can be used
to create the persistent release branch and the tag. The branch and tag needs to
be pushed manually to GitHub. After the tag is pushed, the release can be created
on GitHub.

The `release_publish.sh` script performs the actual publishing to crates.io.
