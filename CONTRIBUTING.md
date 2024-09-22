# Contributing to Eclipse iceoryx2

1. Every new contributor must sign the
   [Eclipse Contributor Agreement (ECA)](https://www.eclipse.org/legal/ECA.php)
   first.
2. Before you start to work, please create an issue first.
3. Create a branch with the prefix `iox2-$ISSUE_NUMBER$`.
4. Every file requires this copyright header.

   ```text
   // Copyright (c) 2023 Contributors to the Eclipse Foundation
   //
   // See the NOTICE file(s) distributed with this work for additional
   // information regarding copyright ownership.
   //
   // This program and the accompanying materials are made available under the
   // terms of the Apache Software License 2.0 which is available at
   // https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
   // which is available at https://opensource.org/licenses/MIT.
   //
   // SPDX-License-Identifier: Apache-2.0 OR MIT
   ```

5. Every commit must have the prefix `[#$ISSUE_NUMBER$]`.
6. When the work is done, please add your changes to the release notes in
   `doc/release-notes/iceoryx2-unreleased.md`.
7. Create a pull request.
8. (optional) If you are a new contributor we would love to show our gratitude
   for your support so please add yourself at the end of `README.md`. The entry
   template looks like this:

   ```html
   <td align="center" valign="top" width="14.28%">
     <a href="https://github.com/$USER_NAME$">
       <img
         src="https://avatars.githubusercontent.com/u/$ID_OF_YOUR_PROFILE_PICTURE$"
         width="120px;"
         alt="$FIRST_NAME$ »$COOL_NICK_NAME$« $LAST_NAME$"
       /><br />
       <sub><b>$FIRST_NAME$ »$COOL_NICK_NAME$« $LAST_NAME$</b></sub></a
     >
   </td>
   ```

   **Notes:**

   * The `$FIRST_NAME »$COOL_NICK_NAME$« $LAST_NAME$` is a suggestion but it can
     be whatever you feel comfortable with.
   * You can obtain the ID of your profile picture by clicking on your profile
     icon on the top right, selecting `Your profile` and then right-clicking on
     your profile picture and selecting `Copy image address`. The last number in
     the URL is the ID of your profile picture.
