# How To Create An iceoryx2 Release

## 0: Start Always With Writing The Articles

### Article Types

1. Write release announcement blog article
2. Write LinkedIn post
3. Write reddit/hacker/programming-dev news post
4. Update the 'ROADMAP.md' document

### Article Template

The link in new release announcement shall always be the link to the
release blog-article.

### Blog Article - Add The Following Links

* Add it at the bottom
  * Discuss on Reddit
  * Discuss on Hacker News
  * Project on GitHub
  * Project on crates.io

### Social Media Post - Add The Following Links

* Add it at the top
  * Release Announcement: <https://ekxide.io/blog/>

* Add it at the bottom
  * repo: <https://github.com/eclipse-iceoryx/iceoryx2>
  * roadmap: <https://github.com/eclipse-iceoryx/iceoryx2/blob/main/ROADMAP.md>
  * crates.io: <https://crates.io/crates/iceoryx2>
  * docs.rs: <https://docs.rs/iceoryx2/latest/iceoryx2>

### Announcement (Major release only)

1. Write blog-article with some technical details, highlights etc.
2. Announce blog-article on
   * <https://www.reddit.com/r/rust/>
   * <https://www.reddit.com/r/programming/>
   * <https://www.reddit.com/r/python/>
   * <https://www.linkedin.com/>
   * <https://news.ycombinator.com/>
   * <https://programming.dev/>
   * <https://techhub.social/>
   * <https://X.com/>
3. If there are interesting things to explore, play around with, post it on
   * <https://news.ycombinator.com/show>

## 1: Check Manual Steps

* Test if QNX builds and runs with the current codebase
* Test if Yocto builds and runs with the current codebase
* check if the new features are marked as done, e.g. `README.md`, `ROADMAP.md`, etc.
* grep for 'planned'
* verify to be on the right branch, e.g. 'main' or 'release-x.y'
* check the code examples in the documentation:
  * `$GIT_ROOT$/README.MD`
  * [iceoryx2 book](https://ekxide.github.io/iceoryx2-book/main/)
    * examples in "Getting Started" section
    * examples in "Tutorials" section

## 2: The Release

There are three scripts to perform the iceoryx2 release

1. `./internal/scripts/release/release_preparation.sh --new-version x.y.z`
2. `./internal/scripts/release/release_tagging.sh`
3. `./internal/scripts/release/release_publish.sh`

### Details

Each script has a `howto` parameter which prints the steps that are performed.

The `release_preparation.sh` script needs to be called with `--new-version x.y.z`.
All the other scripts can be called without arguments.

After the `release_preparation.sh` script is run, a PR needs to be created and
merged to the main branch. Once merged, the `release_tagging.sh` can be used
to create the persistent release branch and the tag. The branch and tag needs to
be pushed manually to GitHub. After the tag is pushed, the release can be created
on GitHub.

The `release_publish.sh` script performs the actual publishing to crates.io.

## Technical Side

1. Use generic release issue ([#77]) and create a new branch
   `iox2-77-X.Y.Z-release`
2. Create new release branch via the GitHub web interface and name it
   `release_X.Y.Z`
3. `just prepare-release all versions --version X.Y.Z`
4. `cargo build --workspace --all-targets` refresh cargo lock file
5. `USE_BAZEL_VERSION=7.4.1 bazel build //...` refresh bazel lock file
6. `peotry --project iceoryx2-ffi/python build-into-venv` refresh poetry lock file
7. `just publish sdk --dry-run --allow-dirty` perform dry run
8. Copy `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` to
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`.
9. Fill out all version place holders (`?.?.?`) in newly created
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`, remove template example
   entries and clean up.
10. Override `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` with
   `$GIT_ROOT$/doc/release-notes/iceoryx2-release-template.md` and bring it in
   the empty state again.
11. **Merge all changes to `main`.**
12. **!! Port reference system to new iceoryx2 version to catch last minute
    bugs !!**
13. **!! Port mission control to new iceoryx2 version to catch last minute
    bugs !!**
14. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
15. `just publish sdk`
16. `just publish integrations`
17. Verify that the release looks fine on `docs.rs` (click through the
    documentation to check if everything was generated correctly)
