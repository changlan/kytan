[![Build Status](https://travis-ci.org/changlan/kytan.svg?branch=master)](https://travis-ci.org/changlan/kytan)
[![codecov](https://codecov.io/gh/changlan/kytan/branch/master/graph/badge.svg)](https://codecov.io/gh/changlan/kytan)

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
$ cargo build --release
```

### Running `kytan`

Like any other VPN server, you need to configure `iptables` to make sure IP
masquerading (or NAT) is enabled. In the future, `kytan` will automate these
steps.

```
$ sudo iptables -t nat -A POSTROUTING -s 10.10.0.0/24 -o eth0 -j MASQUERADE
```

You may change `eth0` to the interface name on your server.
