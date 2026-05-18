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

1. Check the Code examples in the documentation:
   * `$GIT_ROOT$/README.MD`
   * [iceoryx2 book](https://ekxide.github.io/iceoryx2-book/main/)
     * examples in "Getting Started" section
     * examples in "Tutorials" section
2. Use generic release issue ([#77]) and create a new branch
   `iox2-77-X.Y.Z-release`
3. Create new release branch via the GitHub web interface and name it
   `release_X.Y.Z`
4. `just prepare-release all versions --version X.Y.Z`
5. `cargo build --workspace --all-targets` refresh cargo lock file
6. `USE_BAZEL_VERSION=7.4.1 bazel build //...` refresh bazel lock file
7. `peotry --project iceoryx2-ffi/python build-into-venv` refresh poetry lock file
8. `just publish sdk --dry-run --allow-dirty` perform dry run
9. Copy `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` to
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`.
10. Fill out all version place holders (`?.?.?`) in newly created
   `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`, remove template example
   entries and clean up.
11. Override `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` with
   `$GIT_ROOT$/doc/release-notes/iceoryx2-release-template.md` and bring it in
   the empty state again.
12. **Merge all changes to `main`.**
13. **!! Port reference system to new iceoryx2 version to catch last minute
    bugs !!**
14. **!! Port mission control to new iceoryx2 version to catch last minute
    bugs !!**
15. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
16. `just publish sdk`
17. `just publish integrations`
18. Verify that the release looks fine on `docs.rs` (click through the
    documentation to check if everything was generated correctly)
