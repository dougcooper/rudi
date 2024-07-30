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
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast,None).await.unwrap();

    tokio::spawn(async move {
        while let Ok(data) = rx1.recv().await {
            println!("{:?}", data.payload);
        }
    });

    assert_eq!(udp.count(), 1);
}

#[tokio::test]
#[serial]
async fn sharing_sockets_not_allowed() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    udp.subscribe(&unicast,None).await.unwrap();

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
#[case::same_type_same_iface_port((config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6993"),config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6993")),1)]
#[case::same_type_same_iface_diff_port((config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6993"),config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6994")),2)]
#[case::diff_type_same_iface_port((config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6993"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6993")),2)]
#[case::diff_type_same_iface_diff_port((config(CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),"0.0.0.0:6993"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994")),2)]
#[case::same_type_same_iface_port_multicast((config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994"),config(CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),"0.0.0.0:6994")),1)]
#[tokio::test]
#[serial]
async fn test_permutations(#[case] conns: (IpConfigV4, IpConfigV4), #[case] result: usize) {
    let mut udp = UdpManager::default();
    udp.subscribe(&conns.0,None).await.unwrap();
    udp.subscribe(&conns.1,None).await.unwrap();
    assert_eq!(udp.count(), result);
}

#[tokio::test]
#[serial]
async fn test_rx_data_unicast() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast,None).await.unwrap();

    let h = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&unicast).unwrap();
    sock.send(data).await.unwrap();

    let r = h.await.unwrap().unwrap();

    assert_eq!(r.payload, data);
}

#[tokio::test]
#[serial]
async fn test_rx_data_broadcast_all() {
    let mut udp = UdpManager::default();
    let broadcast = IpConfigV4 {
        cast_mode: CastMode::Broadcast,
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&broadcast,None).await.unwrap();

    let h = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&broadcast).unwrap();
    sock.send_to(data,"255.255.255.255:6993").await.unwrap();

    let r = h.await.unwrap().unwrap();

    assert_eq!(r.payload, data);
}

#[tokio::test]
#[serial]
async fn test_cant_broadcast_on_unicast() {
    let mut udp = UdpManager::default();
    let broadcast = IpConfigV4 {
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&broadcast,None).await.unwrap();

    let _ = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&broadcast).unwrap();
    assert!(sock.send_to(data,"255.255.255.255:6993").await.is_err());
}

#[tokio::test]
#[serial]
async fn test_rx_data_multicast() {
    let mut udp = UdpManager::default();
    let mcast = IpConfigV4 {
        cast_mode: CastMode::Multicast("225.1.1.100".parse::<Ipv4Addr>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&mcast,None).await.unwrap();

    let h = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&mcast).unwrap();
    sock.send_to(data,"225.1.1.100:6993").await.unwrap();

    let r = h.await.unwrap().unwrap();

    assert_eq!(r.payload, data);
}

use tokio::runtime::Runtime;

#[test]
#[serial]
fn call_from_sync(){
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let rt = Runtime::new().unwrap();
    let mut rx1 = rt.block_on(udp.subscribe(&unicast,None)).unwrap();

    let h = rt.spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&unicast).unwrap();
    rt.block_on(sock.send_to(data,"127.0.0.1:6993")).unwrap();

    let r = rt.block_on(h).unwrap().unwrap();

    assert_eq!(r.payload, data);
}

#[tokio::test]
#[serial]
async fn test_multiple_rx_data_unicast() {
    let mut udp = UdpManager::default();
    let unicast = IpConfigV4 {
        cast_mode: CastMode::Unicast("127.0.0.1:6993".parse::<SocketAddrV4>().unwrap()),
        addr: "0.0.0.0:6993".parse::<SocketAddrV4>().unwrap(),
    };
    let mut rx1 = udp.subscribe(&unicast,None).await.unwrap();
    let mut rx2 = udp.subscribe(&unicast,None).await.unwrap();

    let h1 = tokio::spawn(async move {
        if let Ok(data) = rx1.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let h2 = tokio::spawn(async move {
        if let Ok(data) = rx2.recv().await {
            Some(data)
        }else{
            None
        }
    });

    let data = b"deadbeef";

    let sock = udp.get_socket(&unicast).unwrap();
    sock.send(data).await.unwrap();

    let (r1,r2) = tokio::join!(h1,h2);

    assert_eq!(r1.unwrap().unwrap().payload, data);
    assert_eq!(r2.unwrap().unwrap().payload, data);
}