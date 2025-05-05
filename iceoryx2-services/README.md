# iceoryx2-services

Being a service-oriented middleware, `iceoryx2` facilitates the ability to add
functionality by spinning up services.

These crates provides some "internal" services that can be used out-of-the-box
to augment applications with some common functionality with minimal hassle.

|      Crate                    | Offered Services             | Description                                        |
|-------------------------------|------------------------------|----------------------------------------------------|
| `iceoryx2-services-discovery` | `iox2://discovery/services/` | Subscribe to service changes in the iceoryx2 system |
