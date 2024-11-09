# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

Create a new CLI for iceoryx2 `iox2-config`

`iox2 config` can `show` the configuration of iceoryx currently in use
or `show` the configuration of the sytem and `generate` a new
configuration file at the default location iceoryx2 is looking for.

* Add CLI to display complete system configuration [#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)

### Refactoring

Remove the `print_system_configuration()` function in
`iceoryx2-bb/posix/src/system_configuration.rs` file and move it into the CLI `iox2-config`
[#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)

### New CLI features

```bash
   iox2-config show system

   iox2-config show current

   iox2-config generate
```
