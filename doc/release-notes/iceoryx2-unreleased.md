# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

* Add CLI to display complete system configuration [#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)

### Refactoring

* Remove the `print_system_configuration()` function in
`iceoryx2-bb/posix/src/system_configuration.rs` file and move it into the CLI `iox2-config`
[#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)

### New CLI features

CLI can show the current iceoryx/system configuration or generate a iceoryx
configuration file.

```sh
iox2 config show system

iox2 config show current

iox2 config generate
```
