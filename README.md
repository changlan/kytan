kytan
=====

[![Build Status](https://travis-ci.org/changlan/kytan.svg?branch=master)](https://travis-ci.org/changlan/kytan)

`kytan` is a peer to peer VPN written in Golang. It is designed to minimize the hassle of configuration
and deployment with a goal of multi-platform support.

Currently, `kytan` can run on Linux as the server, and run on both Linux and Mac OS X as the client.

Usage
-----

Build:

    go get github.com/changlan/kytan
    go build github.com/changlan/kytan

If run as a server, make sure IP forwarding and NAT are enabled:

    sysctl -w net.ipv4.ip_forward=1
    # Assuming the external interface is eth0
    iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE 

Run as a server:

    kytan -mode=server -bind=<PORT> -secret=<SECRET>
    
Run as a client:

    kytan -mode=client -addr=<SERVER_IP> -port=<SERVER_PORT> -secret=<SECRET>
    
`SECRET` can be any string.