`mangi`
=====
`mangi` is a peer to peer VPN written in Golang. It is designed to minimize the hassle of configuration
and deployment with a goal of multi-platform support.

Currently, `mangi` can run on Linux as the server, and run on both Linux and Mac OS X as the client.

Run as a server:

    mangi -mode=server -bind=<PORT> -secret=<SECRET>
    
Run as a client:

    mangi -mode=client -addr=<SERVER_IP> -port=<SERVER_PORT> -secret=<SECRET>
    
`SECRET` can be any string.

Fun fact
--------
Mangi was used to refer to Southern China by the Mongols and North Chinese as a derogation 
(southern barbarians). The word also reached the Western world as Marco Polo used it. However, 
while the North Chinese was placing their bet on the Great Wall for defending nomadic peoples, 
"mangi" began their expedition to the South China Sea and established several settlements.
