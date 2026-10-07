extern crate openrnet;

fn main() {
    let rsock = openrnet::socket::RnetSock::new("vcan0").unwrap();
}
