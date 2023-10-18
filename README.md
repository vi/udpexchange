# udpexchange

This program follows the following algorithm:

1. Listen UDP socket at specified address
2. For each incoming UDP datagram, remember source address in LRU cache
3. Return empty datagrams back to sender (pings/keepalives)
4. Forward non-empty datagrams to each known unexpired address (except of sender).

Optionally, it can remember some recent messages and send them to newly seen clients.

## Security

This service may allow DDoS amplification, so should not be run publicly.

## Small executable size

This project is partly an experiment to create small executables using Rust while having (partial?) access to libstd and using reasonable command-line arguments parser.

    cargo build --release -Zbuild-std=std,panic_abort -Zbuild-std-features=panic_immediate_abort --target=x86_64-unknown-linux-musl --features=mini

should produce a working 51-kilobyte executable.

Note that hacks activated by `--features=mini` may be unsound and less portable. Even smaller size is attainable by also using [eyra](https://docs.rs/crate/eyra/latest).

## Installation

Download a pre-built executable from [Github releases](https://github.com/vi/udpexchange/releases) or install from source code with `cargo install --path .`  or `cargo install udpexchange`.

## CLI options

<details><summary> udpexchange --help output</summary>

```
Usage: udpexchange <listen_addr> [-t <timeout>] [-r]

Simple UDP service which replies to all other known clients

Positional Arguments:
  listen_addr       socket address to bind UDP to

Options:
  -t, --timeout     timeout, in seconds, to expire clients.
  -r, --replay      send recent accumulated messages to newly seen clients
  --help            display usage information
```
</details>
