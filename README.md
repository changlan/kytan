[![Build Status](https://travis-ci.org/changlan/kytan.svg?branch=master)](https://travis-ci.org/changlan/kytan)

## kytan: High Performance Peer-to-Peer VPN

`kytan` is a high performance peer to peer VPN written in Rust. The goal is to
to minimize the hassle of configuration and deployment with a goal of
multi-platform support.

### Supported Platforms

- Linux
- macOS (Client mode only)

### Installation

Currently, `kytan` is in pre-alpha stage. You can compile it from source if
your machine is installed with [Rust](https://www.rust-lang.org/en-US/install.html).

```
$ git clone git@github.com:changlan/kytan.git
$ cd kytan
$ cargo build
```

### Running `kytan`

Like any other VPN server, you need to configure `iptables` so that IP
masquerading (or NAT) is enabled. In the future, `kytan` will automate these
steps.
