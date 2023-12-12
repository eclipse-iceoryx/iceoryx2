## Notes for Reviewer
<!-- Items in addition to the checklist below that the reviewer should look for -->

## Pre-Review Checklist for the PR Author

1. [ ] Add sensible notes for the reviewer
1. [ ] PR title is short, expressive and meaningful
1. [ ] Relevant issues are linked
1. [ ] Every source code file has a copyright header with `SPDX-License-Identifier: Apache-2.0 OR MIT`
1. [ ] Branch follows the naming format (`iox2-123-introduce-posix-ipc-example`)
1. [ ] Commits messages are according to this [guideline][commit-guidelines]
    - [ ] Commit messages have the issue ID (`[#123] Add posix ipc example`)
    - [ ] Commit author matches [Eclipse Contributor Agreement][eca] (and ECA is signed)
1. [ ] Tests follow the [best practice for testing][testing]
1. [ ] Changelog updated [in the unreleased section][changelog] including API breaking changes
1. [ ] Assign PR to reviewer
1. [ ] All checks have passed (except `task-list-completed`)

[commit-guidelines]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
[eca]: http://www.eclipse.org/legal/ECA.php
[testing]: https://github.com/eclipse-iceoryx/iceoryx/blob/master/doc/website/concepts/best-practice-for-testing.md
[changelog]: https://github.com/larry-robotics/iceoryx2/blob/main/doc/release-notes/iceoryx2-unreleased.md

## Checklist for the PR Reviewer

- [ ] Commits are properly organized and messages are according to the guideline
- [ ] Unit tests have been written for new behavior
- [ ] Public API is documented
- [ ] PR title describes the changes

## Post-review Checklist for the PR Author

1. [ ] All open points are addressed and tracked via issues

## References

Use either 'Closes #123' or 'Relates to #123' to reference the corresponding issue.

Closes #**ISSUE-NUMBER**
