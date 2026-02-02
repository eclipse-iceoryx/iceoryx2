# Eclipse IP Due Diligence

As part of the
[Eclipse Due Diligence process](https://www.eclipse.org/projects/handbook/#ip)
, we have to scan our third party content for licenses which are approved by
the Eclipse Foundation. The
[dash-license](https://github.com/eclipse-dash/dash-licenses) tool can be used
for this task.

## Scanning Rust Dependencies

For our Rust code, this can be done with the following command:

```sh
cargo tree -e normal --prefix none --no-dedupe \
      | sort -u \
      | grep -v '^[[:space:]]*$' \
      | grep -v iceoryx2 \
      | grep -v benchmark \
      | grep -v component-tests \
      | grep -v examples \
      | grep -v zenoh \
      | sed -E 's|([^ ]+) v([^ ]+).*|crate/cratesio/-/\1/\2|' \
      | java -jar /path/to/org.eclipse.dash.licenses-x.y.z.jar -
```

If the following output is printed, no further steps are required.

```console
[main] INFO Vetted license information was found for all content. No further investigation is required.
```

In case the tool finds dependencies which are not yet part of the Eclipse license
database, the output looks like this.

```console
[main] INFO Querying Eclipse Foundation for license data for 337 items.
[main] INFO Found 153 items.
[main] INFO Querying ClearlyDefined for license data for 184 items.
[main] INFO Found 184 items.
[main] INFO License information could not be automatically verified for the following content:
[main] INFO
[main] INFO crate/cratesio/-/foo/0.8.15
[main] INFO
[main] INFO This content is either not correctly mapped by the system, or requires review.
```

In this case, the dependency must be reported to the IP team. For more details,
take a look at [Reporting New Dependencies](#reporting-new-dependencies).

## Reporting New Dependencies

In case the license check could not find the information for a third party
component, an issue must be created at
<https://gitlab.eclipse.org/eclipsefdn/emo-team/iplab>. This can be done
automatically with the `dash` license tool. For details, please have a look at
<https://github.com/eclipse-dash/dash-licenses?tab=readme-ov-file#automatic-ip-team-review-requests>

For Rust dependencies, this is the command to execute:

```sh
cargo tree -e normal --prefix none --no-dedupe \
      | sort -u \
      | grep -v '^[[:space:]]*$' \
      | grep -v iceoryx2 \
      | grep -v benchmark \
      | grep -v component-tests \
      | grep -v examples \
      | grep -v zenoh \
      | sed -E 's|([^ ]+) v([^ ]+).*|crate/cratesio/-/\1/\2|' \
      | java -jar /path/to/org.eclipse.dash.licenses-x.y.z.jar -review -project technology.iceoryx -token [TOKEN] -
```

## Listing Dependencies

In order to document the dependencies, e.g. in NOTICE.md, the `dash` tool can
also be used to list all dependencies.

```sh
cargo tree -e normal --prefix none --no-dedupe \
      | sort -u \
      | grep -v '^[[:space:]]*$' \
      | grep -v iceoryx2 \
      | grep -v benchmark \
      | grep -v component-tests \
      | grep -v examples \
      | grep -v zenoh \
      | sed -E 's|([^ ]+) v([^ ]+).*|crate/cratesio/-/\1/\2|' \
      | java -jar /path/to/org.eclipse.dash.licenses-x.y.z.jar -summary summary-rust.txt -
```
