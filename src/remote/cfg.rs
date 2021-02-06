use std::{io, net::SocketAddr, sync::Arc};

use crate::cfg::{self, ConfigItem, ConfigMap};

#[derive(Debug)]
pub struct Config {
    addr: SocketAddr,
    key: Arc<Vec<u8>>,
    hash: Arc<Vec<u8>>,
    time_out_sec: u64,
}

impl Config {
    pub async fn read() -> io::Result<Self> {
        let map = ConfigMap::read_config_file().await?;

        let addr = map
            .get(ConfigItem::ServerAddr)
            .expect("❌配置文件找不到server_addr项")
            .parse()
            .expect("配置文件server_addr项转换socket_addr失败");

        let key = map
            .get(ConfigItem::Key)
            .expect("❌配置文件找不到key项")
            .as_bytes()
            .to_vec();

        let hash = cfg::to_hash(&key).as_bytes().to_vec();

        let time_out_sec = map
            .get(ConfigItem::TimeOutSec)
            .expect("❌配置文件找不到time_out_sec项")
            .parse()
            .expect("配置文件time_out_sec项转换u64失败");

        Ok(Self {
            addr,
            key: Arc::new(key),
            hash: Arc::new(hash),
            time_out_sec,
        })
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn key(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.key)
    }

    pub fn hash(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.hash)
    }

    pub fn time_out_sec(&self) -> u64 {
        self.time_out_sec
    }
}
