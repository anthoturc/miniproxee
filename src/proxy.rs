use std::net;

pub struct Peer {
    addr: net::IpAddr,
}

#[derive(Default)]
pub struct MiniProxee {
    upstream_peers: Vec<Peer>,
}
