use std::{collections::HashMap, sync::Arc};

use tokio::{io, sync::broadcast::{self, Receiver}};

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
            let (tx, _) = broadcast::channel::<Datagram>(channel_size.unwrap_or(16));
            let c = Connection::new(&config,tx).await?;
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
