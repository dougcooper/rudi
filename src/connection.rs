use crate::{Datagram, IpConfigV4};
use socket2::{Domain, SockAddr, Socket, Type, Protocol};
use tokio::io;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::{self, Receiver, Sender};

const MAX_PDU_SIZE_BYTES: usize = 8192;

fn make_udp_socket(addr: &SockAddr, _reuse_port: bool) -> io::Result<Socket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    sock.set_nonblocking(true)?;
    sock.bind(addr)?;
    Ok(sock)
}

pub struct Connection {
    tx: Sender<Datagram>,
}

impl Connection {
    pub async fn new(config: &IpConfigV4) -> io::Result<Self> {
        let s = make_udp_socket(
            &SockAddr::from(config.addr),
            //we dont want to allow port reuse for unicast, otherwise another listener on the same port could steal data
            match config.cast_mode{
                crate::CastMode::Unicast => false,
                _=> true,
            }
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
        tokio::spawn(async move {
            let mut buf = [0; MAX_PDU_SIZE_BYTES];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok(_) => {
                        if tx1.receiver_count() > 0 {
                            if let Err(_) = tx1.send(buf.into()) {
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
        Ok(Connection { tx: tx })
    }

    pub fn subscribe(&self) -> Receiver<Datagram> {
        self.tx.subscribe()
    }
}
