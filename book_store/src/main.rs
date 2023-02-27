use std::env;
use xml_rpc::server::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

fn main() {
    let args: Vec<String> = env::args().collect();

    let server_name = match args.get(1) {
        Some(a) => a.to_string() + &".cs.williams.edu:8013".to_string(),
        None => panic!("Enter a server name!"),
    };

    let ip_addr: SocketAddr = match server_name.to_socket_addrs().expect("Bad Server Name").next() {
        Some(a) => a,
        None => panic!("Server did not match an ip addr!"),
    };

    let server: Server = Server::new();
    let response = server.bind(&ip_addr);
}
