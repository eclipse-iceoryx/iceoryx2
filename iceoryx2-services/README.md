# iceoryx2-services

As a service-oriented middleware, `iceoryx2` is designed to extend application
capabilities through composable services.

This directory contains ready-to-use service crates that provide common
functionality on top of the core `iceoryx2` infrastructure. Each crate can be
integrated into applications with minimal effort.

<!-- markdownlint-disable MD060 -->

| Crate                         | Offered Services             | Description                                                        |
| ----------------------------- | ---------------------------- | ------------------------------------------------------------------ |
| `iceoryx2-services-discovery` | `iox2://discovery/services/` | Receive notifications when services are created, changed or removed |
| `iceoryx2-services-tunnel`    | -                            | Extend  `iceoryx2` communication over a network connection          |

<!-- markdownlint-enable MD060 -->
