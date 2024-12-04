<!-- markdownlint-disable MD013 Line breaks on the bullet list lines are also present on the github renderer, therefore no line length limitation -->
<!-- markdownlint-disable MD041 On the github PR template we want to start with '## Headline' -->

## Notes for Reviewer
<!-- Items in addition to the checklist below that the reviewer should look for -->

## Pre-Review Checklist for the PR Author

* [ ] Add sensible notes for the reviewer
* [ ] PR title is short, expressive and meaningful
* [ ] Consider switching the PR to a draft (`Convert to draft`)
    * as draft PR, the CI will be skipped for pushes
* [ ] Relevant issues are linked in the [References](#references) section
* [ ] Every source code file has a copyright header with `SPDX-License-Identifier: Apache-2.0 OR MIT`
* [ ] Branch follows the naming format (`iox2-123-introduce-posix-ipc-example`)
* [ ] Commits messages are according to this [guideline][commit-guidelines]
    * [ ] Commit messages have the issue ID (`[#123] Add posix ipc example`)
    * [ ] Commit author matches [Eclipse Contributor Agreement][eca](and ECA is signed)
* [ ] Tests follow the [best practice for testing][testing]
* [ ] Changelog updated [in the unreleased section][changelog] including API breaking changes
* [ ] Assign PR to reviewer
* [ ] All checks have passed (except `task-list-completed`)

[commit-guidelines]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
[eca]: http://www.eclipse.org/legal/ECA.php
[testing]: https://github.com/eclipse-iceoryx/iceoryx/blob/master/doc/website/concepts/best-practice-for-testing.md
[changelog]: https://github.com/eclipse-iceoryx/iceoryx2/blob/main/doc/release-notes/iceoryx2-unreleased.md

## Checklist for the PR Reviewer

* [ ] Commits are properly organized and messages are according to the guideline
* [ ] Unit tests have been written for new behavior
* [ ] Public API is documented
* [ ] PR title describes the changes

## Post-review Checklist for the PR Author

* [ ] All open points are addressed and tracked via issues

## References

<!-- Use either 'Closes #123' or 'Relates to #123' to reference the corresponding issue. -->

Closes # <!-- Add issue number after '#' -->

<!-- markdownlint-enable MD041 -->
<!-- markdownlint-enable MD013 -->
