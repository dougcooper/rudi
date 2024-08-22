use std::{collections::HashMap, sync::Arc};

use tokio::sync::broadcast::Receiver;
use tokio::io;

use crate::{connection::Connection, Datagram, IpConfigV4};

#[derive(Default)]
pub struct UdpManager {
    connections: HashMap<IpConfigV4, Connection>,
}

impl UdpManager {
    pub async fn subscribe(&mut self, ip_config: &IpConfigV4, channel_size: Option<usize>) -> io::Result<Receiver<Datagram>> {

        let conn = if let Some(conn) = self.connections.get(ip_config) {
            conn
        } else {
            let (tx,_) = if let Some(size) = channel_size {
                // async_broadcast::broadcast::<Datagram>(size)
                tokio::sync::broadcast::channel(size)
            }else{
                // async_broadcast::broadcast::<Datagram>(u16::MAX as usize)
                tokio::sync::broadcast::channel(u16::MAX as usize)
            };
            let c = Connection::new(&ip_config,tx).await?;
            self.connections.insert(ip_config.clone(), c);
            self.connections.get(ip_config).unwrap()
        };

        Ok(conn.subscribe())
    }

    pub fn count(&self) -> usize {
        self.connections.len()
    }

    pub fn get_socket(&self, config: &IpConfigV4) -> Option<Arc<tokio::net::UdpSocket>> {
        if let Some(conn) = self.connections.get(config){
            Some(conn.socket.clone())
        }else{
            None
        }
    } 
}
