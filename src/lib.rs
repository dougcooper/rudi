use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub mod connection;
pub mod udpmanager;

#[derive(Clone)]
pub struct Datagram {
    pub payload: Vec<u8>,
    pub sender: SocketAddr
}

#[derive(PartialEq,Eq,Hash,Clone,Debug)]
pub struct MulticastConfig {
    pub group: Ipv4Addr,
    pub interface: Ipv4Addr
}

#[derive(PartialEq,Eq,Hash,Clone,Debug)]
pub enum CastMode {
    Unicast(SocketAddrV4),
    Broadcast,
    Multicast(MulticastConfig)
}

#[derive(PartialEq,Eq,Hash,Clone,Debug)]
pub struct IpConfigV4{
    pub cast_mode: CastMode,
    pub bind_addr: SocketAddrV4
}

#[cfg(test)]
mod test{
    use std::collections::HashMap;

    use crate::*;

    #[test]
    fn it_can_hash(){
        let config = IpConfigV4{
            cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
            bind_addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap()
        };

        let mut m = HashMap::new();
        m.insert(config.clone(), 1);
        m.insert(config.clone(), 2);

        assert_eq!(*m.get(&config).unwrap(),2);
        assert_eq!(m.len(),1);
    }
}