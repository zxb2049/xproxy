use std::{
    fmt::Display,
    io::{self, Error, ErrorKind},
    str::FromStr,
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::util;

pub struct XRequest(Vec<u8>);

impl XRequest {
    pub fn new(mut host: String, key: &[u8], hash: &str) -> Self {
        let host = unsafe { host.as_bytes_mut() };
        util::encrypt(host, key, 0);

        let mut inner = hash.as_bytes().to_vec();
        inner.append(&mut host.to_vec());

        Self(inner)
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }

    pub async fn read(src: &mut TcpStream, hash: &[u8], time_out: Duration) -> io::Result<Self> {
        let mut buf = [0; 512];
        let len = timeout(time_out, src.read(&mut buf))
            .await
            .map_err(|_| Error::new(ErrorKind::TimedOut, "🔴磨磨唧唧，恕不招待"))??;

        match buf[..len].starts_with(hash) {
            true => {
                let inner = buf[..len].to_vec();
                Ok(Self(inner))
            }
            false => Err(Error::new(ErrorKind::Other, "🔴XRequest的hash校验失败")),
        }
    }

    pub fn to_host(self, key: &[u8], hash: &[u8]) -> String {
        let mut host = self.0[hash.len()..].to_vec();
        util::encrypt(&mut host, key, 0);

        String::from_utf8_lossy(&host).to_string()
    }
}
#[derive(Copy, Clone)]
pub enum XResponse {
    Ok,
    NotFound,
    BadHash,
}

impl Display for XResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ok => "🤪肉蛋葱鸡",
                Self::NotFound => "😵歪比歪比，歪比巴卜",
                Self::BadHash => "🤔你是神魔捏？你在干神魔",
            },
        )
    }
}

impl FromStr for XResponse {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [Self::Ok, Self::NotFound, Self::BadHash]
            .iter()
            .find(|x| x.to_string() == s)
            .map(|x| *x)
            .ok_or(Error::new(ErrorKind::Other, "🔴无法解析XResponse"))
    }
}

impl XResponse {
    pub async fn is_ok(src: &mut TcpStream, dst: &mut TcpStream) -> io::Result<()> {
        let mut buf = [0; 256];
        let len = dst.read(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..len]);

        match msg.parse()? {
            Self::Ok => Ok(()),
            Self::NotFound => {
                src.write_all(b"HTTP/1.1 404 NotFound\r\n\r\n").await?;
                Err(Error::new(ErrorKind::NotFound, "🔴无法连接指定地址"))
            }
            Self::BadHash => panic!("❌口令或哈希错误"),
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}
