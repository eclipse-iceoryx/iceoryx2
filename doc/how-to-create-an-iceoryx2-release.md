# How To Create An iceoryx2 Release

## Technical Side

Assume that the new version number is `X.Y.Z`.

 1. Use generic release issue ([#77]) and create a new branch `iox2-77-X.Y.Z-release`
 2. Copy `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` to
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`.
 3. Fill out all version place holders/old version numbers in newly created
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`, remove template example
    entries and clean up.
 4. Add the section `Thanks To All Contributors Of This Version` in
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md` and list all contributors
    of the new release.
 5. Add new long-term contributors to the `$GIT_ROOT$/README.md`.
    * Shall have provided multiple PRs and reviews/issues.
 6. Override
    `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` with
    `$GIT_ROOT$/doc/release-notes/iceoryx2-release-template.md`
    and bring it in the empty state again.
 7. (Major release only) Create `$GIT_ROOT$/doc/announcements/iceoryx2-vX.Y.Z.md`
    and fill it with all the different announcement texts.
 8. Change `workspace.package.version` in `$GIT_ROOT$/Cargo.toml` to the new
    version number `X.Y.Z`.
    * **IMPORTANT** change version to `X.Y.Z` for all `iceoryx2-**` packages under
      `[workspace.dependencies]`
 9. **Merge all changes to `main`.**
 10. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
 11. Call `$GIT_ROOT$/./internal/scripts/crates_io_publish_script.sh` and publish
    all crates on `crates.io` and `docs.rs`.
 12. Verify that the release looks fine on `docs.rs`
    (click through the documentation to check if everything was generated
    correctly)

## Announcement (Major release only)

 1. Write blog-article with some technical details, highlights etc.
 2. Announce blog-article on
    * https://www.reddit.com/r/rust/
    * https://www.linkedin.com/
    * https://news.ycombinator.com/
    * https://techhub.social/
    * https://X.com/
 3. If there are interesting things to explore, play around with, post it on
    * https://news.ycombinator.com/show
