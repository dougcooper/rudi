use serial_test::serial;
use tokio::net::UdpSocket;
use std::net::{Ipv4Addr, SocketAddrV4};
use rudi::{udpmanager::UdpManager, CastMode, IpConfigV4};

//tests have the `serial` attribute so they dont fail due to port conflicts

#[tokio::test]
#[serial]
async fn example_usage() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast,
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast).await.unwrap();

    tokio::spawn(async move {
        while let Ok(data) = rx1.recv().await {
            println!("{:?}", data);
        }
    });

    assert_eq!(udp.count(), 1);
}

#[tokio::test]
#[serial]
async fn sharing_sockets_not_allowed() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast,
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    udp.subscribe(&unicast).await.unwrap();

    assert!(UdpSocket::bind("0.0.0.0:6993").await.is_err());
}

use rstest::*;

fn config(mode: CastMode, addr: &str) -> IpConfigV4 {
    IpConfigV4 {
        cast_mode: mode,
        addr: addr.parse::<SocketAddrV4>().unwrap(),
    }
}

#[rstest]
#[case::same_type_same_iface_port((config(CastMode::Unicast,"0.0.0.0:6993"),config(CastMode::Unicast,"0.0.0.0:6993")),1)]
#[case::same_type_same_iface_diff_port((config(CastMode::Unicast,"0.0.0.0:6993"),config(CastMode::Unicast,"0.0.0.0:6994")),2)]
#[case::diff_type_same_iface_port((config(CastMode::Unicast,"0.0.0.0:6993"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6993")),2)]
#[case::diff_type_same_iface_diff_port((config(CastMode::Unicast,"0.0.0.0:6993"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994")),2)]
#[case::same_type_same_iface_port_multicast((config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994")),1)]
#[tokio::test]
#[serial]
async fn test_permutations(#[case] conns: (IpConfigV4, IpConfigV4), #[case] result: usize) {
    let mut udp = UdpManager::default();
    udp.subscribe(&conns.0).await.unwrap();
    udp.subscribe(&conns.1).await.unwrap();
    assert_eq!(udp.count(), result);
}

#[tokio::test]
#[serial]
async fn test_rx_data_unicast() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast,
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast).await.unwrap();

    let h = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&unicast).unwrap();
    sock.send_to(data,"127.0.0.1:6993").await.unwrap();

    let r = h.await.unwrap().unwrap();

    assert_eq!(r, data);
}

#[tokio::test]
#[serial]
async fn test_rx_data_multicast() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast).await.unwrap();

    let h = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&unicast).unwrap();
    sock.send_to(data,"225.1.1.100:6993").await.unwrap();

    let r = h.await.unwrap().unwrap();

    assert_eq!(r, data);
}