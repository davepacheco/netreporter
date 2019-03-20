# netreporter

netreporter is (will be) a half-baked tool for monitoring basic connectivity of
a network.  It's aimed at recording data for understanding home network
problems, but there's nothing specific to home networks.


## Status

Still vaporware.


## Goals

First, this is a toy project, partly intended to learn Rust.

My goal is to see graphs of connectivity within my home network as well as from
my home network to WAN endpoints (e.g., `8.8.8.8`) so that I can help understand
cases where the network is not working well.  To make it concrete, I'd like to:

- easily stand up netreporter server instances on components to monitor
- support basic connectivity tests (e.g., ICMP ping, plus UDP and TCP analogs)
  among netreporter instances
- support basic short throughput tests over TCP among netreporter instances

Ideally, on my home network, I'd do this:

- set up a netreporter instance on computer A on the LAN (wired)
- set up a netreporter instance on an external endpoint B (across the WAN)
- run periodic connectivity checks between:
  - A and the LAN router (ICMP ping)
  - A and the LAN cable modem (ICMP ping)
  - A and WAN endpoint (ICMP ping, netreporter-level UDP/TCP)
  - A and B (ICMP ping, netreporter-level UDP/TCP)
- run periodic throughput checks between A and B (netreporter-level TCP)

The intent is to expose these all using Prometheus endpoints built into
netreporter, then feed that into Grafana.  With this set-up (which should be
reasonably easy to orchestrate), one could identify windows of packet loss or
high latency and determine whether it's related to the local network or the WAN
connection.

I could also set up another endpoint on the LAN called C and run connectivity
and throughput checks between A and C.  In theory, one could set up multiple
endpoints on different parts of the WAN to assess internet connectivity, but I
don't have enough expertise to know whether this would be a reliable approach
(and I'm not interested in that use-case).

It would be nice to be able to set up ad-hoc instances (e.g., from a laptop on
wifi) so that when running into a new network issue you can start collecting
data immediately in an organized way.

I want to support OS X and SmartOS, since those are what I run on endpoints.


## Netreporter

netreporter is (will be) a server program that supports:

- configuration to periodically run tests against remote endpoints, including:
  - ICMP ping checks against other ICMP endpoints
  - a basic UDP ping check using other netreporter instances
  - a basic TCP ping check using other netreporter instances
  - a basic TCP throughput test using other netreporter instances
- listening for TCP connections and UDP packets from other netreporter instances
  to participate in any of the above TCP and UDP checks
- listening for TCP connections for reporting stats in Prometheus format


## Plan

- build a command-line tool for sending an ICMP ping packet and getting a
  response
- build a pair of command-line tools for sending a basic TCP ping and responding
  to it (and reporting on the latency)
- add Prometheus support
  - figure out how to expose metrics so that it's easy to get historical graphs
- prototype the whole system: Prometheus, Grafana, plus 1-2 netreporter
  instances
- add configuration for periodic runs of these various checks
- add basic TCP throughput test
