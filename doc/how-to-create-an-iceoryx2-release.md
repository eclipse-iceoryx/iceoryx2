# How To Create An iceoryx2 Release

Assume that the new version number is `X.Y.Z`.

 1. Change `workspace.package.version` in `$GIT_ROOT$/Cargo.toml` to the new
    version number `X.Y.Z`.
 2. Copy `$GIT_ROOT$/doc/iceoryx2-unreleased.md` to
    `$GIT_ROOT$/doc/iceoryx2-vX.Y.Z.md`.
 3. Fill out all version place holders/old version numbers in newly created
    `$GIT_ROOT$/doc/iceoryx2-vX.Y.Z.md`, remove template example entries and
    clean up.
 4. Add the section `Thanks To All Contributors Of This Version` in
    `$GIT_ROOT$/doc/iceoryx2-vX.Y.Z.md` and list all contributors of the
    new release.
 5. Add new long-term contributors to the `$GIT_ROOT$/README.md`.
    * Shall have provided multiple PRs and reviews/issues.
 6. Merge all changes to `main`.
 7. Call `$GIT_ROOT$/./internal/scripts/crates_io_publish_script.sh` and publish
    all crates on `crates.io` and `docs.rs`.
 8. Verify that the release looks fine on `docs.rs`
    (click through the documentation to check if everything was generated
    correctly)
 9. Set tag on GitHub and add the release document as notes to the tag
    description. Add also a link to the file.
 10. Announce new release on:
    * https://www.reddit.com/r/rust/
    * https://www.linkedin.com/
