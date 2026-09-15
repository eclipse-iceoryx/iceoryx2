# iceoryx2-link

> [!IMPORTANT]
> The link is currently a prototype and requires validation in real
> deployments. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

The crates of the link, which extends `iceoryx2` services across the
boundary of a shared memory domain.

* To use a link, read [`link`](link/README.md).
* To provide a carrier, read its contract crate and check it against
  `conformance-tests`.

## Crates

| Crate | Contents |
| ----- | -------- |
| `link` | The core, local discovery, bridges, propagation. |
| `backend` | The contract between a link and its backend, and the types crossing it. |
| `carrier` | The contract a mechanism implements to carry a tunnel. |
| `tunnel` | The tunnel backend, `iceoryx2` to `iceoryx2` over a carrier. |
| `testing` | Fake substitutes for a carrier. |
| `conformance-tests` | The suites a carrier or a backend is checked against. |
