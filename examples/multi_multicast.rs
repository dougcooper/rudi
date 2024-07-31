use std::net::{Ipv4Addr, SocketAddrV4};
use async_broadcast::Receiver;
use clap::Parser;
use rudi::{udpmanager::UdpManager, CastMode, Datagram, IpConfigV4, MulticastConfig};

async fn recv_data(rx: &mut Receiver<Datagram>, name: &str) {
    while let Ok(data) = rx.recv().await {
        println!("received {} bytes of data from {name}", data.payload.len());
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Multicast group to join
    #[arg(short, long, default_value_t = String::from("224.1.1.100"))]
    group: String,

    /// Interface to join multicast group on
    #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
    interface: String,

    /// Interface and Port to bind socket to
    #[arg(short, long, default_value_t = String::from("0.0.0.0:6993"))]
    destination: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut udp = UdpManager::default();
    let mcast = IpConfigV4 {
        cast_mode: CastMode::Multicast(MulticastConfig{
            group: args.group.parse::<Ipv4Addr>().unwrap(),
            interface: args.interface.parse::<Ipv4Addr>().unwrap(),
        }),
        bind_addr: args.destination.parse::<SocketAddrV4>().unwrap(),
    };

    let mut rx1 = udp.subscribe(&(mcast.clone()),None).await.unwrap();
    let mut rx2 = udp.subscribe(&(mcast.clone()),None).await.unwrap();

    println!("{:?}",args);

    let mut ctrlc = tokio::spawn(tokio::signal::ctrl_c());

    loop {
        tokio::select! {
            _ = &mut ctrlc => break,
            _ = recv_data(&mut rx1, "rx1") => {}
            _ = recv_data(&mut rx2, "rx2") => {}
        }
    }

    println!("ctrlc called. exiting...");
}
