use std::sync::Arc;

use crate::{Datagram, IpConfigV4};
use async_broadcast::{Receiver, Sender};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::io;
use tokio::net::UdpSocket;

const MAX_DATAGRAM_SIZE: usize = 65507;

fn make_udp_socket(addr: &SockAddr, _reuse_port: bool) -> io::Result<Socket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    sock.set_nonblocking(true)?;
    sock.bind(addr)?;
    Ok(sock)
}

pub struct Connection {
    rx: Receiver<Datagram>,
    pub socket: Arc<UdpSocket>,
}

impl Connection {
    pub async fn new(ip_config: &IpConfigV4, tx: Sender<Datagram>,rx: Receiver<Datagram>) -> io::Result<Self> {
        let s = make_udp_socket(
            &SockAddr::from(ip_config.bind_addr),
            //we dont want to allow port reuse for unicast, otherwise another listener on the same port could steal data
            match ip_config.cast_mode {
                crate::CastMode::Unicast(_) => false,
                _ => true,
            },
        )?;

        let socket = UdpSocket::from_std(s.into())?;

        match &ip_config.cast_mode {
            crate::CastMode::Unicast(addr) => {
                socket.connect(addr.to_string()).await?;
            }
            crate::CastMode::Broadcast => {
                socket.set_broadcast(true)?;
            }
            crate::CastMode::Multicast(mcast_config) => {
                socket.join_multicast_v4(mcast_config.group, mcast_config.interface)?;
            }
        };

        let socket_rx = Arc::new(socket);
        let socket_tx = socket_rx.clone();
        let cast_mode = ip_config.cast_mode.clone();
        tokio::spawn(async move {
            let mut buf = [0u8;MAX_DATAGRAM_SIZE];
            loop {
                match cast_mode {
                    crate::CastMode::Unicast(addr) => {
                        match socket_rx.recv(&mut buf).await {
                            Ok(bytes_read) => {
                                if tx.receiver_count() > 0 && bytes_read > 0 {
                                    let data = if let Some(data) = buf.get(..bytes_read) {
                                        data.into()
                                    } else {
                                        vec![]
                                    };
                                    if let Err(_) = tx.broadcast(Datagram { payload: data.into(), sender: addr.into() }).await {
                                        //TODO: log error
                                    };
                                }
                            }
                            Err(_) => {
                                //TODO: log error
                            }
                        }
                    },
                    crate::CastMode::Multicast(_) | crate::CastMode::Broadcast=> {
                        match socket_rx.recv_from(&mut buf).await {
                            Ok((bytes_read, sender)) => {
                                if tx.receiver_count() > 0 && bytes_read > 0 {
                                    let data = if let Some(data) = buf.get(..bytes_read) {
                                        data.into()
                                    } else {
                                        vec![]
                                    };
                                    if let Err(_) = tx.broadcast(Datagram { payload: data.into(), sender: sender }).await {
                                        //TODO: log error
                                    };
                                }
                            }
                            Err(_) => {
                                //TODO: log error
                            }
                        }
                    },
                }
            }
        });
        Ok(Connection {
            rx: rx,
            socket: socket_tx,
        })
    }

    pub fn subscribe(&self) -> Receiver<Datagram> {
        self.rx.clone()
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    #[test]
    fn it_works() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"12345"[..]);
        assert!(buf.get(..5).is_some());
        assert!(buf.get(..6).is_none());
    }

    #[tokio::test]
    async fn test_async_channel() {
        let (tx,rx) = async_broadcast::broadcast::<u32>(5);

        let mut rx1 = rx.clone();
        let mut rx2 = rx;

        tx.broadcast(1).await.unwrap();

        let (r1,r2) = tokio::join!(rx1.recv(),rx2.recv());

        assert_eq!(r1.unwrap(),1);
        assert_eq!(r2.unwrap(),1);
    }

}
