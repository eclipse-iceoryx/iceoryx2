# How To Create An iceoryx2 Release

## Start Always With Writing The Articles

1. Write release announcement blog article
2. Write LinkedIn post
3. Write reddit/hacker news post
4. Update the `ROADMAP.md` document

### Article Templates

Link in new release announcement shall always be the link to the release
blog-article.

#### Blog Article - Add The Following Links

```text
[Add it at the top]

 * Discuss on Reddit
 * Discuss on Hacker News
 * Project on GitHub
 * Project on crates.io
```

#### Social Media Post - Add The Following Links

```text
[Add it at the top]
 * Release Announcement: https://ekxide.io/blog/****************

[Add it at the bottom]
 * repo: https://github.com/eclipse-iceoryx/iceoryx2
 * roadmap: https://github.com/eclipse-iceoryx/iceoryx2/blob/main/ROADMAP.md
 * crates.io: https://crates.io/crates/iceoryx2
 * docs.rs: https://docs.rs/iceoryx2/latest/iceoryx2
```

### Announcement (Major release only)

1. Write blog-article with some technical details, highlights etc.
2. Announce blog-article on
   * <https://www.reddit.com/r/rust/>
   * <https://www.linkedin.com/>
   * <https://news.ycombinator.com/>
   * <https://techhub.social/>
   * <https://X.com/>
3. If there are interesting things to explore, play around with, post it on
   * <https://news.ycombinator.com/show>

## Technical Side

Assume that the new version number is `X.Y.Z` and the previous version
number is `Xold.Yold.Zold`.

1. Check the Code examples in the documentation:
   * `$GIT_ROOT$/README.MD`
   * `$GIT_ROOT$/internal/cpp_doc_generator/*.rst`
2. Use generic release issue ([#77]) and create a new branch
   `iox2-77-X.Y.Z-release`
3. Copy `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` to
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`.
4. Fill out all version place holders (`?.?.?`) in newly created
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`, remove template example
   entries and clean up.
5. Override `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` with
   `$GIT_ROOT$/doc/release-notes/iceoryx2-release-template.md` and bring it in
   the empty state again.
6. Change `workspace.package.version` in `$GIT_ROOT$/Cargo.toml` to the new
   version number `X.Y.Z`.
   * **IMPORTANT** change version to `X.Y.Z` for all `iceoryx2-**` packages
     under `[workspace.dependencies]`
7. Adjust the `<version>` to `X.Y.Z` in `$GIT_ROOT$/package.xml`.
8. Call `rg "Xold\.Yold\.Zold"` and adjust all findings.
    * C and C++ examples, `BUILD.bazel` & `CMakeLists.txt`
9. Adjust the major, minor and patch version number in
    `iceoryx2_bb/elementary/package_version.rs` in `PackageVersion::get()`
10. Add the release notes to `$GIT_ROOT$/CHANGELOG.md`
11. **Merge all changes to `main`.**
12. **!! Port reference system to new iceoryx2 version to catch last minute
    bugs !!**
13. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
14. Check the order of all dependencies in
    `$GIT_ROOT$/./internal/scripts/crates_io_publish_script.sh`. Compare it
    with `$GIT_ROOT$/Cargo.toml` and the `[workspace.dependencies]` section.
    When calling `cargo publish -p $PACKAGE$` all dependencies, also
    dev-dependencies, must be already published to `crates.io` via
    `cargo publish -p`. Verify the
    correct ordering of dependencies by checking the `[dependencies]` and
    `[dev-dependencies]`
    section in the `Cargo.toml` file of every crate in the workspace.
    * If the publish script was started and a crate requires a dependency which
      is not available on `crates.io` the release has to be redone and the patch
      version has to increase by one for the whole workspace.
15. Call `$GIT_ROOT$/./internal/scripts/crates_io_publish_script.sh` and publish
    all crates on `crates.io` and `docs.rs`.
16. Verify that the release looks fine on `docs.rs` (click through the
    documentation to check if everything was generated correctly)
