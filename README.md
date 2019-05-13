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

Currently, precompiled `kytan` binaries are available for Linux and macOS.
You can download them from [releases](https://github.com/changlan/kytan/releases).

Alternatively, you can compile it from source if
your machine is installed with [Rust](https://www.rust-lang.org/en-US/install.html).

```
$ git clone https://github.com/changlan/kytan.git
$ cd kytan
$ cargo build --release
```

### Running `kytan`

For complete information:

```
$ sudo ./kytan -h
```

#### Server Mode

Like any other VPN server, you need to configure `iptables` as following to make
sure IP masquerading (or NAT) is enabled, which should be done only once. In the
future, `kytan` will automate these steps. You may change `<INTERFACE>` to the
interface name on your server (e.g. `eth0`):

```
$ sudo iptables -t nat -A POSTROUTING -s 10.10.10.0/24 -o <INTERFACE> -j MASQUERADE
```

To run `kytan` in server mode and listen on UDP port `9527` with password `hello`:

```
$ sudo ./kytan server -k hello 
```
If you want open log display (`info` is log level, you can change it by your idea)

```
$ sudo RUST_LOG=info ./kytan server -k hello 
```

#### Client Mode

To run `kytan` in client mode and connect to the server `<SERVER>:9527` using password `hello`:

```
$ sudo ./kytan client -s <SERVER> -p 9527 -k hello
```

if you want open log display (`info` is log level, you can change it by your idea)

```
$ sudo RUST_LOG=info ./kytan client -s <SERVER> -p 9527 -k hello
```

### License

Apache 2.0
