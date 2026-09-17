# iox2-link-tunnel-zenoh

> [!IMPORTANT]
> The link is currently a prototype and requires validation in real
> deployments. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

CLI running the link's tunnel between `iceoryx2` systems over zenoh. The
tunnel assumes the systems it joins and the network between them are
trusted. See the [`iceoryx2-link-tunnel`](../../../iceoryx2-link/tunnel/src/lib.rs)
crate's documentation for details.

## Usage

```bash
cargo install iceoryx2-integrations-zenoh-link-tunnel-cli
iox2 link tunnel zenoh --help
```
