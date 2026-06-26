# miniproxee

A lightweight TCP proxy implementation written in Rust.

## Why

I have interviews that I need to study for and the best way to prep
for them is to write a proxy implementation. This repository
wasn't created using AI because that would sort of defeat the purpose.

## Requirements

1. Proxy all TCP connections between downstream and upstream peers
2. Detect whether the HTTP protocol is being used and route traffic through configurable middleware
3. Apply different upstream peer selection options. Round robin and consistent hashing based on IP. Configurable
4. Report per-connection metrics
5. Allow for hooking into open telemetry so that proxy performance can be viewed
6. Handle backpressure correctly from both ends of the pipe
7. Allow for TLS to be applied when configured.

## Design

### Main

The artifact of this repository is a binary that can proxy TCP connections 
between peers on the internet. Configuration options can be specified on 
the command line via flags.

### Proxy

The proxy itself will be implemented as a [tower service](https://docs.rs/tower/latest/tower/).

It will allow a configurable list of upstream peers. For simplicity, the proxy isn't performing service discovery
or looking up DNS. Instead, the upstream peers are hardcoded and specified via a config file.

### HTTP Detection

Similar to linkerd, the incoming TCP stream will be used to determine if the request is HTTP. A timeout of 10 seconds
is put in place to avoid hanging and waiting for bytes on the other side.

## Running, Testing, Building

### Testing

A `docker-compose.yml` is available in the test directory. You can run `docker-compose up -d` to start up a series
of test upstream servers. All they do is echo whatever input is supplied to them. They run over tcp.

