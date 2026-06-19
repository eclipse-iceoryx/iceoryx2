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
* check if the new features are marked as done, e.g. `README.md`, `ROADMAP.md`,
  etc.
* grep for 'planned'
* verify to be on the right branch, e.g. 'main' or 'release-x.y'
* check the code examples in the documentation:
    * `$GIT_ROOT$/README.MD`
    * [iceoryx2 book](https://ekxide.github.io/iceoryx2-book/main/)
        * examples in "Getting Started" section
        * examples in "Tutorials" section
* **!! Port reference system to new iceoryx2 version to catch last minute
    bugs !!**
* **!! Port mission control to new iceoryx2 version to catch last minute
    bugs !!**

## 2: The Release

There are three scripts to perform the iceoryx2 release

1. `./internal/scripts/release/release_preparation.sh --new-version x.y.z`
2. Create pull request and merge the release branch to `main`
3. `./internal/scripts/release/release_tagging.sh`
4. Login to [crates.io](https://crates.io) and generate API token
5. `export CARGO_REGISTRY_TOKEN ???`
6. `./internal/scripts/release/release_publish.sh`
7. Verify that the release looks fine on `docs.rs` (click through the
    documentation to check if everything was generated correctly)
8. For patch releases, port the changelog back to the main branch and set
   `PREVIOUS_RELEASE` from `internal/VERSIONS` to the latest patch release

## 3: Publish Python bindings to pypi.org

1. Start a [release workflow](https://github.com/eclipse-iceoryx/iceoryx2/actions/workflows/release.yml)
   for the new tag
2. Once finished, the `python-dist-...` artifact needs to be downloaded and
   extracted
3. If not yet done, create a API token at <https://pypi.org>
4. Upload the dist artifacts with the following command. During the execution
   of the command, the API token is requested

```sh
poetry --project=path/to/iceoryx2-ffi/python/ upload-dist --dist /full/path/to/extracted/dist/artifacts --repo pypi
```

Reference for configuration: <https://packaging.python.org/en/latest/specifications/pypirc/>

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
