use std::{io, net::SocketAddr, sync::Arc};

use crate::cfg::{self, ConfigItem, ConfigMap};

#[derive(Debug)]
pub struct Config {
    addr: SocketAddr,
    server_addr: Arc<SocketAddr>,
    key: Arc<Vec<u8>>,
    hash: Arc<String>,
}

impl Config {
    pub async fn read() -> io::Result<Self> {
        let map = ConfigMap::read_config_file().await?;

        let addr = map
            .get(ConfigItem::ClientAddr)
            .expect("❌配置文件找不到client_addr项")
            .parse()
            .expect("❌配置文件client_addr项转换socket_addr失败");

        let server_addr = map
            .get(ConfigItem::ServerAddr)
            .expect("❌配置文件找不到server_addr项")
            .parse()
            .expect("❌配置文件server_addr项转换socket_addr失败");

        let key = map
            .get(ConfigItem::Key)
            .expect("❌配置文件找不到key项")
            .as_bytes()
            .to_vec();

        let hash = cfg::to_hash(&key);

        Ok(Self {
            addr,
            server_addr: Arc::new(server_addr),
            key: Arc::new(key),
            hash: Arc::new(hash),
        })
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn server_addr(&self) -> Arc<SocketAddr> {
        Arc::clone(&self.server_addr)
    }

    pub fn key(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.key)
    }

    pub fn hash(&self) -> Arc<String> {
        Arc::clone(&self.hash)
    }
}
