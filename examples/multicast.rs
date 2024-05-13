use std::net::{Ipv4Addr, SocketAddrV4};
use rudi::{udpmanager::UdpManager, CastMode, Datagram, IpConfigV4};

async fn recv_data(rx: &mut tokio::sync::broadcast::Receiver<Datagram>) {
    while let Ok(data) = rx.recv().await {
        println!("received {} bytes of data", data.payload.len());
    }
}

#[tokio::main]
async fn main() {
    let mut udp = UdpManager::default();
    let mcast = IpConfigV4 {
        cast_mode: CastMode::Multicast("224.1.1.100".parse::<Ipv4Addr>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&mcast,None).await.unwrap();

    let mut ctrlc = tokio::spawn(tokio::signal::ctrl_c());

    loop {
        tokio::select! {
            _ = &mut ctrlc => break,
            _ = recv_data(&mut rx1) => {}
        }
    }

    println!("ctrlc called. exiting...");
}