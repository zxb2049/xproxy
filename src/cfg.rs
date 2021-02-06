use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
    io::{self, ErrorKind},
};

use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub fn to_hash(key: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(key);

    hasher.finish().to_string()
}

const CONFIG_FILE_PATH: &str = "config";
const DEFAULT_CONFIG: &str = r#"client_addr = 127.0.0.1:7878
key = 芜湖，起飞~🚀
server_addr = 127.0.0.1:8989
time_out_sec = 3"#;

pub struct ConfigMap(HashMap<String, String>);

impl ConfigMap {
    pub async fn read_config_file() -> io::Result<Self> {
        let mut file = match OpenOptions::new().read(true).open(CONFIG_FILE_PATH).await {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                Self::create_default_config_file().await?;
                panic!("❌找不到配置文件，已创建默认配置文件");
            }
            Err(e) => return Err(e),
        };

        let mut content = String::new();
        file.read_to_string(&mut content).await?;

        let inner = content
            .lines()
            .map(|line| line.splitn(2, " = ").collect::<Vec<&str>>())
            .filter(|v| v.len() == 2)
            .map(|v| (v[0], v[1]))
            .map(|(key, val)| (key.to_string(), val.to_string()))
            .collect();

        Ok(Self(inner))
    }

    async fn create_default_config_file() -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(CONFIG_FILE_PATH)
            .await?;

        file.write_all(DEFAULT_CONFIG.as_bytes()).await?;
        Ok(())
    }

    pub fn get(&self, item: ConfigItem) -> Option<&String> {
        let key = match item {
            ConfigItem::ClientAddr => "client_addr",
            ConfigItem::Key => "key",
            ConfigItem::ServerAddr => "server_addr",
            ConfigItem::TimeOutSec => "time_out_sec",
        };

        self.0.get(key)
    }
}

pub enum ConfigItem {
    ServerAddr,
    ClientAddr,
    Key,
    TimeOutSec,
}
