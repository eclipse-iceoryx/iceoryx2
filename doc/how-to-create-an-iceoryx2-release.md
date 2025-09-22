# How To Create An iceoryx2 Release

There are three scripts to perform the iceoryx2 release

* internal/scripts/release_preparation.sh
* internal/scripts/release_tagging.sh
* internal/scripts/release_publish.sh

Each script has a `howto` parameter which prints the steps that are performed.

The `release_preparation.sh` script needs to be called with `--new-version x.y.z`.
All the other scripts can be called without arguments.

After the `release_preparation.sh` script is run, a PR needs to be created and
merged to the main branch. Once merged, the `release_tagging.sh` can be used
to crate the persistent release branch and the tag. The branch and tag needs to
be pushed manually to github. After the tag is pushed to github, the release on
github can be created.

The `release_publish.sh` script performs the actual publishing to crates.io.
