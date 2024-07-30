use std::{collections::HashMap, sync::Arc};

use async_broadcast::Receiver;
use tokio::io;

use crate::{connection::Connection, Datagram, IpConfigV4};

#[derive(Default)]
pub struct UdpManager {
    connections: HashMap<IpConfigV4, Connection>,
}

impl UdpManager {
    pub async fn subscribe(&mut self, config: &IpConfigV4, channel_size: Option<usize>) -> io::Result<Receiver<Datagram>> {

        let conn = if let Some(conn) = self.connections.get(config) {
            conn
        } else {
            let (tx,rx) = if let Some(size) = channel_size {
                async_broadcast::broadcast::<Datagram>(size)
            }else{
                async_broadcast::broadcast::<Datagram>(u16::MAX as usize)
            };
            let c = Connection::new(&config,tx,rx).await?;
            self.connections.insert(config.clone(), c);
            self.connections.get(config).unwrap()
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
