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
* To provide a carrier or an adapter implementation, follow the instructions
  in the documentation for the corresponding crate and verify the implementation
  against the `conformance-tests`.

## Crates

| Crate | Contents |
| ----- | -------- |
| `link` | The core, local discovery, bridges, propagation. |
| `backend` | The contract between a link and its backend, a tunnel or a gateway. |
| `tunnel` | The tunnel backend, connecting `iceoryx2` to `iceoryx2` via a carrier. |
| `carrier` | The contract a mechanism implements to carry the bytes of the tunnel backend. |
| `gateway` | The gateway backend, `iceoryx2` to a middleware over an adapter. |
| `adapter` | The contract a middleware implements to integrate with the gateway backend. |
| `testing` | Fake substitutes for a carrier and for a middleware. |
| `conformance-tests` | The suites a carrier, an adapter or a backend contracts are checked against. |
