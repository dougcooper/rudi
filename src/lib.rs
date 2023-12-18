use std::net::{Ipv4Addr, SocketAddrV4};

mod connection;
pub mod udpmanager;

#[derive(Clone,Debug)]
pub struct Datagram(Vec<u8>);

#[derive(PartialEq,Eq,Hash,Clone)]
pub enum CastMode {
    Unicast,
    Broadcast,
    Multicast(Ipv4Addr)
}

#[derive(PartialEq,Eq,Hash,Clone)]
pub struct IpConfigV4{
    pub cast_mode: CastMode,
    pub addr: SocketAddrV4
}

#[cfg(test)]
mod test{
    use std::collections::HashMap;

    use crate::*;

    #[test]
    fn if_can_hash(){
        let config = IpConfigV4{
            cast_mode: CastMode::Unicast,
            addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap()
        };

        let mut m = HashMap::new();
        m.insert(config.clone(), 1);
        m.insert(config.clone(), 2);

        assert_eq!(*m.get(&config).unwrap(),2);
        assert_eq!(m.len(),1);
    }
}