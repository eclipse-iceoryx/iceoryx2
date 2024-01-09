# How To Create An iceoryx2 Release

Assume that the new version number is `X.Y.Z`.

 1. Copy `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` to
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`.
 2. Fill out all version place holders/old version numbers in newly created
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md`, remove template example
    entries and clean up.
 3. Add the section `Thanks To All Contributors Of This Version` in
    `$GIT_ROOT$/doc/release-notes/iceoryx2-vX.Y.Z.md` and list all contributors
    of the new release.
 4. Add new long-term contributors to the `$GIT_ROOT$/README.md`.
    * Shall have provided multiple PRs and reviews/issues.
 5. Remove all entries from
    `$GIT_ROOT$/doc/release-notes/iceoryx2-unreleased.md` and bring it in the
    empty state again. Fill it with example entries.
 6. Create `$GIT_ROOT$/doc/announcements/iceoryx2-vX.Y.Z.md` and fill it with
    all the different announcement texts.
 7. **Merge all changes to `main`.**

 8. Change `workspace.package.version` in `$GIT_ROOT$/Cargo.toml` to the new
    version number `X.Y.Z`.
    * **IMPORTANT** change version to `X.Y.Z` for all `iceoryx2-**` packages under
      `[workspace.dependencies]`
 9. Call `$GIT_ROOT$/./internal/scripts/crates_io_publish_script.sh` and publish
    all crates on `crates.io` and `docs.rs`.
 10. Verify that the release looks fine on `docs.rs`
    (click through the documentation to check if everything was generated
    correctly)
 11. **Merge all changes to `main`.**

 12. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
 13. Announce new release on:
    * https://www.reddit.com/r/rust/
    * https://www.linkedin.com/
