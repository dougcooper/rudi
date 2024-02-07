use std::sync::Arc;

use crate::{Datagram, IpConfigV4};
use bytes::BytesMut;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::{self, Receiver, Sender};

const MAX_DATAGRAM_SIZE: usize = 65507;

fn make_udp_socket(addr: &SockAddr, _reuse_port: bool) -> io::Result<Socket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    sock.set_nonblocking(true)?;
    sock.bind(addr)?;
    Ok(sock)
}

pub struct Connection {
    tx: Sender<Datagram>,
    pub socket: Arc<UdpSocket>,
}

impl Connection {
    pub async fn new(config: &IpConfigV4) -> io::Result<Self> {
        let s = make_udp_socket(
            &SockAddr::from(config.addr),
            //we dont want to allow port reuse for unicast, otherwise another listener on the same port could steal data
            match config.cast_mode {
                crate::CastMode::Unicast => false,
                _ => true,
            },
        )?;

        let socket = UdpSocket::from_std(s.into())?;

        match config.cast_mode {
            crate::CastMode::Unicast => {}
            crate::CastMode::Broadcast => {
                socket.set_broadcast(true)?;
            }
            crate::CastMode::Multicast(addr) => {
                socket.join_multicast_v4(addr, config.addr.ip().clone())?;
            }
        };

        //TODO: how big should this be
        let (tx, _) = broadcast::channel::<Datagram>(16);
        let tx1 = tx.clone();
        let socket_rx = Arc::new(socket);
        let socket_tx = socket_rx.clone();
        tokio::spawn(async move {
            let mut buf = BytesMut::with_capacity(MAX_DATAGRAM_SIZE);
            loop {
                match socket_rx.recv_from(&mut buf).await {
                    Ok((bytes_read, _)) => {
                        if tx1.receiver_count() > 0 && bytes_read > 0 {
                            let data = if let Some(data) = buf.get(..bytes_read - 1) {
                                data.into()
                            } else {
                                vec![]
                            };
                            if let Err(_) = tx1.send(data.into()) {
                                //TODO: log error
                            };
                        }
                    }
                    Err(_) => {
                        //TODO: log error
                    }
                }
            }
        });
        Ok(Connection {
            tx: tx,
            socket: socket_tx,
        })
    }

    pub fn subscribe(&self) -> Receiver<Datagram> {
        self.tx.subscribe()
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
}
